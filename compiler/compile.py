#!/usr/bin/env python3


import sys

import attr

from s_expression_parser import parse, ParserConfig, Pair, nil


# Data Destinations
# DD_A..DD_HL are registers.
# DD_TMP refers to the temporary stack of intermediate results.
#   Note that DD_DE represents the top of that stack.
#   Other values are spilled into RAM.

DD_A  = 0
DD_BC = 1
DD_DE = 2
DD_HL = 3
DD_TMP = 4
DD_ZFLAG = 5

# Control Destinations
# RET implies a return after the current expression or statement.
# Control destinations >= CD_LABEL refer to locally generated labels.
CD_RET = 0
CD_NEXT = 1
CD_LABEL = 100

def starts_with_decimal_digit(t):
    return '0' <= t[0] <= '9'


def is_octal(t):
    return t[-1] in ['o', 'O']

def is_hex(t):
    return t[-1] in ['h', 'H']

def is_decimal(t):
    return (not is_octal(t)) and (not is_hex(t))


def is_pair(node):
    return isinstance(node, Pair)


def to_number(t):
    if len(t) >= 1:
        if t[0] == '0':
            if len(t) >= 3:
                if t[1] in ['x', 'X']:
                    return int(t[2:], 16)
                elif t[1] in ['o', 'O']:
                    return int(t[2:], 8)
                elif t[1] in ['b', 'B']:
                    return int(t[2:], 2)
            return int(t, 8)
        elif '1' <= t[0] <= '9':
            return int(t, 10)
    else:
        raise ValueError("to_number called with empty token")


class Compiler:
    def __init__(self):
        self.parser_config = ParserConfig({}, dots_are_cons=True)
        self.assembly_listing = None
        self.globals = list()
        self.next_label = CD_LABEL - 1

    def make_label(self):
        self.next_label = self.next_label + 1
        return self.next_label

    def main(self, script=None):
        self.assembly_listing = []

        tree = parse(script, self.parser_config)
        for node in tree:
            self.cg_form(node, DD_HL, CD_RET)
        for line in self.assembly_listing:
            print(line)

    def cg_form(self, node, dd, cd):
        if dd == DD_ZFLAG:
            self.cg_form(node, DD_HL, CD_NEXT)
            self.asm(None, "LD", "A,L")
            self.asm(None, "OR", "A,H")
            self.cg_goto(cd)
        elif is_pair(node):
            if node.car == '+':
                self.cg_binop(self.cg_add, node, dd, cd)
            elif node.car == '-':
                self.cg_binop(self.cg_subtract, node, dd, cd)
            elif node.car == '*':
                self.cg_binop(self.cg_multiply, node, dd, cd)
            elif node.car == '/':
                self.cg_binop(self.cg_divide, node, dd, cd)
            elif node.car == '&':
                self.cg_binop(self.cg_bit_and, node, dd, cd)
            elif node.car == '|':
                self.cg_binop(self.cg_bit_or, node, dd, cd)
            elif node.car == '^':
                self.cg_binop(self.cg_bit_xor, node, dd, cd)
            elif node.car == 'int16':
                self.declare_variables(node)
            elif node.car == 'set':
                self.cg_set_var(node, dd, cd)
            elif node.car == 'if':
                self.cg_if(node, dd, cd)
            elif node.car == 'sub':
                self.cg_sub(node, dd, CD_RET)
            elif node.car == 'do':
                self.cg_statements(node.cdr, dd, CD_RET)
            else:
                if node.car not in self.globals:
                    raise ValueError("Unsupported: {}".format(node.car))
                else:
                    if node.cdr is not nil:
                        raise ValueError("Arguments to subroutines not supported: {}".format(node.car))
                    else:
                        self.cg_call_libfn(node.car, cd)
        else:
            if starts_with_decimal_digit(node):
                n = to_number(node)

                if dd in [DD_BC, DD_DE, DD_HL]:
                    self.cg_ld16(dd, n)
                    self.cg_goto(cd)
                else:
                    raise ValueError("Unknown data destination: {}".format(dd))
            else:
                if node in self.globals:
                    self.cg_ld16_gv(dd, node)
                    self.cg_goto(cd)
                else:
                    raise ValueError("Symbol not declared: {}".format(node))

    def cg_statements(self, node, dd, cd):
        return_handled = False
        while node is not nil:
            if node.cdr is not nil:
                next_target = DD_HL
                next_step = CD_NEXT
            else:
                next_target = dd
                next_step = cd
                return_handled = True
            self.cg_form(node.car, next_target, next_step)
            node = node.cdr
        if not return_handled:
            self.cg_goto(cd)

    def cg_sub(self, node, dd, cd):
        # (sub NAME S1 S2 ...)
        name = node.cdr.car
        statements = node.cdr.cdr
        if name in self.globals:
            raise ValueError("Symbol already defined: {}".format(name))
        self.globals.append(name)
        self.cg_emit_label(name)
        self.cg_statements(statements, dd, cd)

    def cg_if(self, node, dd, cd):
        # (if PRED CONSEQ ALTERopt)
        label_false = self.make_label()
        label_end = self.make_label()

        pred = node.cdr.car
        conseq = node.cdr.cdr.car
        alter = None
        if node.cdr.cdr.cdr is not nil:
            alter = node.cdr.cdr.cdr.car

        if alter is None:
            if cd != CD_RET:
                self.cg_form(pred, DD_ZFLAG, (CD_NEXT, label_false))
                self.cg_form(conseq, DD_HL, cd)
                self.cg_emit_label(label_false)
                self.cg_goto(cd)
            else:
                self.cg_form(pred, DD_ZFLAG, (CD_NEXT, cd))
                self.cg_form(conseq, DD_HL, cd)
        else:
            self.cg_form(pred, DD_ZFLAG, (CD_NEXT, label_false))
            self.cg_form(conseq, DD_HL, label_end)
            self.cg_emit_label(label_false)
            self.cg_form(alter, DD_HL, CD_NEXT)
            self.cg_emit_label(label_end)
            self.cg_goto(cd)

    def declare_variables(self, node):
        varlist = node.cdr
        while varlist is not nil:
            if varlist.car in self.globals:
                raise ValueError("Variable already defined: {}".format(varlist.car))
            self.globals.append(varlist.car)
            self.asm(varlist.car, "DEFW", "0")
            varlist = varlist.cdr

    def cg_set_var(self, node, dd, cd):
        # (set VAR EXPR_hl)
        v = node.cdr.car
        e = node.cdr.cdr.car

        self.cg_form(e, DD_HL, CD_NEXT)
        self.asm(None, "LD", "({}),HL".format(v))
        self.cg_ld16_r16(dd, DD_HL)
        self.cg_goto(cd)

    def cg_binop(self, op, node, dd, cd):
        if is_pair(node.cdr.car):
            self.cg_form(node.cdr.cdr.car, DD_HL, CD_NEXT)
            self.cg_push_hl()
            self.cg_form(node.cdr.car, DD_HL, CD_NEXT)
            self.cg_pop_de()
        else:
            self.cg_form(node.cdr.cdr.car, DD_DE, CD_NEXT)
            self.cg_form(node.cdr.car, DD_HL, CD_NEXT)
        op(dd, DD_HL, DD_DE, cd)

    def cg_ld16_gv(self, dd, t):
        self.asm(None, "LD", "{},({})".format(self.to_reg(dd), t))

    def cg_op16(self, dd, ds1, ds2, cd, op1, op2):
        self.asm(None, "LD", "A,{}".format(self.to_reg(ds1)[1]))
        self.asm(None, op1, "A,{}".format(self.to_reg(ds2)[1]))
        self.asm(None, "LD", "{},A".format(self.to_reg(dd)[1]))
        self.asm(None, "LD", "A,{}".format(self.to_reg(ds1)[0]))
        self.asm(None, op2, "A,{}".format(self.to_reg(ds2)[0]))
        self.asm(None, "LD", "{},A".format(self.to_reg(dd)[0]))
        self.cg_goto(cd)

    def cg_add(self, dd, ds1, ds2, cd):
        def do_add(d, a, b):
            if (d == DD_HL) and (a == DD_HL):
                self.asm(None, "ADD", "HL,{}".format(self.to_reg(b)))
            else:
                self.cg_op16(dd, ds1, ds2, cd, "ADD", "ADC")

        if (dd == DD_HL) and (ds2 == DD_HL):
            do_add(dd, ds2, ds1)
        else:
            do_add(dd, ds1, ds2)
        self.cg_goto(cd)

    def cg_bit_and(self, dd, ds1, ds2, cd):
        self.cg_op16(dd, ds1, ds2, cd, 'AND', 'AND')

    def cg_bit_or(self, dd, ds1, ds2, cd):
        self.cg_op16(dd, ds1, ds2, cd, 'OR', 'OR')

    def cg_bit_xor(self, dd, ds1, ds2, cd):
        self.cg_op16(dd, ds1, ds2, cd, 'XOR', 'XOR')

    def cg_subtract(self, dd, ds1, ds2, cd):
        self.cg_op16(dd, ds1, ds2, cd, 'SUB', 'SBC')

    def cg_divide(self, dd, ds1, ds2, cd):
        self.cg_call_libfn("divide_{}_{}".format(self.to_reg(ds1), self.to_reg(ds2)), cd)
        self.cg_ld16_r16(dd, DD_HL)

    def cg_multiply(self, dd, ds1, ds2, cd):
        self.cg_call_libfn("multiply_{}_{}".format(self.to_reg(ds1), self.to_reg(ds2)), cd)
        self.cg_ld16_r16(dd, DD_HL)

    def cg_call_libfn(self, fn_name, cd):
        if cd != CD_RET:
            self.asm(None, "CALL", fn_name)
            self.cg_goto(cd)
        else:
            self.asm(None, "JP", fn_name)

    def cg_op_pair(self, op1, op2, dd, ds):
        src = self.to_reg(ds)
        dst = self.to_reg(dd)
        self.asm(None, op1, "{},{}".format(dst[1], src[1]))
        self.asm(None, op2, "{},{}".format(dst[0], src[0]))

    def cg_ld16(self, dd, n):
        self.asm(None, "LD", "{},{}".format(self.to_reg(dd), n))

    def cg_ld16_r16(self, dd, ds):
        if ds == dd:
            return
        self.cg_op_pair("LD", "LD", dd, ds)

    def cg_ld8_r8(self, rd, rs):
        self.asm(None, "LD", "{},{}".format(rd, rs))

    def cg_goto(self, cd):
        if isinstance(cd, tuple):
            true_branch = cd[0]
            false_branch = cd[1]

            if true_branch == CD_NEXT:
                if false_branch == CD_NEXT:
                    pass
                elif false_branch == CD_RET:
                    self.asm(None, "RET", "Z")
                else:
                    self.asm(None, "JP", "Z,L{}".format(false_branch))
            elif true_branch == CD_RET:
                if false_branch == CD_NEXT:
                    self.asm(None, "RET", "NZ")
                elif false_branch == CD_RET:
                    self.cg_goto(CD_RET)
                else:
                    self.asm(None, "RET", "NZ")
                    self.cg_goto(false_branch)
        elif cd == CD_NEXT:
            pass
        elif cd == CD_RET:
            self.asm(None, "RET", None)
        elif cd >= CD_LABEL:
            self.asm(None, "JP", "L{}".format(cd))
        else:
            raise ValueError("Unknown control destination: {}".format(cd))

    def cg_push_hl(self):
        self.asm(None, "PUSH", "HL")

    def cg_push_de(self):
        self.asm(None, "PUSH", "DE")

    def cg_pop_de(self):
        self.asm(None, "POP", "DE")

    def cg_emit_label(self, l):
        if l is not None:
            if isinstance(l, int):
                l = "L{}".format(l)
            self.assembly_listing.append("{}:".format(l))

    def asm(self, label, mnem, oper):
        self.cg_emit_label(label)
        if oper is None:
            oper = ""
        self.assembly_listing.append("    {:6} {}".format(mnem, oper))

    def to_reg(self, dd):
        return {
            DD_BC: 'BC',
            DD_DE: 'DE',
            DD_HL: 'HL',
        }[dd]


if __name__ == '__main__':
    Compiler().main(open(sys.argv[1]).read())

