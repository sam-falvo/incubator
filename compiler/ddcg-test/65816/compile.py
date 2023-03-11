#!/usr/bin/env python3


import sys

import attr

from s_expression_parser import parse, ParserConfig, Pair, nil


# Data Destinations
# DD_AC, DD_XR, DD_YR are registers.
# DD_ZFLAG is for effect only.
# DD_1S refers to the top of the stack (effective address is 1,S).

DD_AC = 0
DD_XR = 1
DD_YR = 2
DD_ZFLAG = 3
DD_1S = 4
DD_ZP = 5

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
        self.zpptr = 0

    def alloc_zp(self):
        self.zpptr = self.zpptr + 2
        return self.zpptr

    def free_zp(self):
        self.zpptr = self.zpptr - 2

    def make_label(self):
        self.next_label = self.next_label + 1
        return self.next_label

    def main(self, script=None):
        self.assembly_listing = []

        tree = parse(script, self.parser_config)
        for node in tree:
            self.cg_form(node, DD_AC, CD_RET)
        for line in self.assembly_listing:
            print(line)

    def cg_form(self, node, dd, cd):
#       if dd == DD_ZFLAG:
#           self.cg_form(node, DD_AC, CD_NEXT)
#           self.asm(None, "ORA", "#0")
#           self.cg_goto(cd)
        if dd == DD_XR:
            self.cg_form(node, DD_AC, CD_NEXT)
            self.asm(None, "TAX", None)
            self.cg_goto(cd)
        elif dd == DD_YR:
            self.cg_form(node, DD_AC, CD_NEXT)
            self.asm(None, "TAY", None)
            self.cg_goto(cd)
        elif dd == DD_ZP:
            self.cg_form(node, DD_AC, CD_NEXT)
            self.asm(None, "STA", "${:02X}".format(self.zpptr - 2))
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
            elif node.car == '@':
                self.cg_address_of(node, dd, cd)
            elif node.car == 'poke':
                self.cg_poke(node, dd, cd)
            elif node.car == 'peek':
                self.cg_peek(node, dd, cd)
            elif node.car == 'output': # Intel/Z80 specific
                self.cg_poke(node, dd, cd)
            elif node.car == 'input': # Intel/Z80 specific
                self.cg_peek(node, dd, cd)
            elif node.car == 'highbyte':
                self.cg_highbyte(node, dd, cd)
            elif node.car == 'lowbyte':
                self.cg_lowbyte(node, dd, cd)
            elif node.car == '>>':
                self.cg_shift_right_logical(node, dd, cd)
            elif node.car == '<<':
                self.cg_shift_left(node, dd, cd)
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

                if dd in [DD_AC, DD_XR, DD_YR]:
                    self.cg_ld16(dd, n)
                    self.cg_goto(cd)
                else:
                    raise ValueError("Unknown data destination: {}".format(dd))
            elif node[0] == '-':
                n = -to_number(node[1:])

                if dd in [DD_AC, DD_XR, DD_YR]:
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

    def cg_bit_or(self, dd, ds1, ds2, cd):
        self._cg_bit_op("ORA", dd, ds1, ds2, cd)

    def cg_bit_and(self, dd, ds1, ds2, cd):
        self._cg_bit_op("AND", dd, ds1, ds2, cd)

    def cg_bit_xor(self, dd, ds1, ds2, cd):
        self._cg_bit_op("EOR", dd, ds1, ds2, cd)

    def cg_add(self, dd, ds1, ds2, cd):
        self.asm(None, "CLC", None)
        self._cg_bit_op("ADC", dd, ds1, ds2, cd)

    def cg_subtract(self, dd, ds1, ds2, cd):
        self.asm(None, "SEC", None)
        self._cg_bit_op("SBC", dd, ds1, ds2, cd)

    def _cg_bit_op(self, op, dd, ds1, ds2, cd):
        if ds1 != DD_AC:
            raise ValueError("ds1 expected to be DD_AC")

        pop = (dd != DD_1S) and (ds2 == DD_1S)
        push = (dd == DD_1S) and (ds2 != DD_1S)

        if ds2 == DD_AC:
            pass
        elif ds2 == DD_XR:
            self.asm(None, "STX", "zt0")
            self.asm(None, op, "zt0")
        elif ds2 == DD_YR:
            self.asm(None, "STY", "zt0")
            self.asm(None, op, "zt0")
        elif ds2 == DD_ZFLAG:
            raise ValueError("DD_AC, DD_ZFLAG doesn't make sense")
        elif ds2 == DD_1S:
            if pop:
                self.asm(None, "PLX", None)
                self.asm(None, "STX", "zt0")
                self.asm(None, op, "zt0")
            else:
                self.asm(None, op, "1,S")
        elif ds2 == DD_ZP:
            self.asm(None, op, "${:02X}".format(self.zpptr - 2))
        else:
            raise ValueError("ds2 Unrecognized destination {}".format(ds2))

        if dd == DD_AC:
            pass
        elif dd == DD_XR:
            self.asm(None, "TAX", None)
        elif dd == DD_YR:
            self.asm(None, "TAY", None)
        elif dd == DD_ZFLAG:
            pass # flags already set
        elif dd == DD_1S:
            if push:
                self.asm(None, "PHA", None)
            else:
                pass
        elif dd == DD_ZP:
            self.asm(None, "STA", "${:02}".format(self.zpptr - 2))


        self.cg_goto(cd)

    def cg_shift_right_logical(self, node, dd, cd):
        # (>> EXPR COUNT)
        e = node.cdr.car
        cnt = node.cdr.cdr.car
        
        n_cnt = None
        if starts_with_decimal_digit(cnt):
            n_cnt = to_number(cnt)

        def do_variable_shift():
            loopback = self.make_label()
            skipahead = None

            self.cg_form(cnt, DD_XR, CD_NEXT)
            if n_cnt is None:
                skipahead = self.make_label()
                self.asm(None, "CPX", "#0")
                self.asm(None, "BEQ", "L{}".format(skipahead))
            self.asm(loopback, "LSR", "A")
            self.asm(None, "DEX", None)
            self.asm(None, "BNE", "L{}".format(loopback))
            if n_cnt is None:
                self.cg_emit_label(skipahead)

        self.cg_form(e, DD_AC, CD_NEXT)
        if n_cnt is None:
            do_variable_shift()
        elif n_cnt > 5:
            do_variable_shift()
        else: # 0 <= n_cnt <= 5:
            for x in range(n_cnt):
                self.asm(None, "LSR", "A")
        self.cg_goto(cd)

    def cg_shift_left(self, node, dd, cd):
        self.asm(None, "<< NOT IMPLEMENTED", None)
        self.cg_goto(cd)

    def cg_highbyte(self, node, dd, cd):
        self.cg_form(node.cdr.car, DD_AC, CD_NEXT)
        self.asm(None, "XBA", None)
        self.asm(None, "AND", "#$00FF")
        self.cg_goto(cd)

    def cg_lowbyte(self, node, dd, cd):
        self.cg_form(node.cdr.car, DD_AC, CD_NEXT)
        self.asm(None, "AND", "#$00FF")
        self.cg_goto(cd)

    def cg_peek(self, node, dd, cd):
        # (peek SIZE ADDR)
        sz = node.cdr.car
        addr = node.cdr.cdr.car

        if dd != DD_AC:
            raise ValueError("Unknown destination {}".format(dd))

        if sz == 'byte':
            self.cg_form(addr, DD_XR, CD_NEXT)
            self.asm(None, "LDA", "0,X")
            self.asm(None, "AND", "#$00FF")
        elif sz == 'word':
            self.cg_form(addr, DD_XR, CD_NEXT)
            self.asm(None, "LDA", "0,X")
        else:
            raise ValueError("Unsupported poke size: {}".format(sz))

        self.cg_goto(cd)

    def cg_poke(self, node, dd, cd):
        # (poke SIZE ADDR DATUM)
        sz = node.cdr.car
        addr = node.cdr.cdr.car
        datum = node.cdr.cdr.cdr.car

        if sz == 'byte':
            self.cg_form(addr, DD_XR, CD_NEXT)
            self.cg_form(datum, DD_AC, CD_NEXT)
            self.asm(None, "SEP", "#$20")
            self.asm(None, "STA", "0,X")
            self.asm(None, "REP", "#$20")
            self.cg_goto(cd)
        elif sz == 'word':
            self.cg_form(addr, DD_XR, CD_NEXT)
            self.cg_form(datum, DD_AC, CD_NEXT)
            self.asm(None, "STA", "0,X")
            self.cg_goto(cd)
        else:
            raise ValueError("Unsupported poke size: {}".format(sz))

    def cg_address_of(self, node, dd, cd):
        # (@ VAR)
        v = node.cdr
        if v is nil:
            raise ValueError("@ operator missing variable or procedure name")
        elif v.car not in self.globals:
            raise ValueError("@ operator reference to undeclared variable or procedure: {}".format(v.car))
        else:
            self.cg_ld16(dd, v.car)
            self.cg_goto(cd)

    def cg_statements(self, node, dd, cd):
        return_handled = False
        while node is not nil:
            if node.cdr is not nil:
                next_target = DD_AC
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
                self.cg_form(conseq, DD_AC, cd)
                self.cg_emit_label(label_false)
                self.cg_goto(cd)
            else:
                self.cg_form(pred, DD_ZFLAG, (CD_NEXT, cd))
                self.cg_form(conseq, DD_AC, cd)
        else:
            self.cg_form(pred, DD_ZFLAG, (CD_NEXT, label_false))
            self.cg_form(conseq, DD_AC, label_end)
            self.cg_emit_label(label_false)
            self.cg_form(alter, DD_AC, CD_NEXT)
            self.cg_emit_label(label_end)
            self.cg_goto(cd)

    def declare_variables(self, node):
        varlist = node.cdr
        while varlist is not nil:
            if varlist.car in self.globals:
                raise ValueError("Variable already defined: {}".format(varlist.car))
            self.globals.append(varlist.car)
            self.asm(varlist.car, ".WORD", "0")
            varlist = varlist.cdr

    def cg_set_var(self, node, dd, cd):
        # (set VAR EXPR_hl)
        v = node.cdr.car
        e = node.cdr.cdr.car

        if dd != DD_AC:
            raise ValueError("Unknown destination {}".format(dd))

        self.cg_form(e, DD_AC, CD_NEXT)
        self.asm(None, "STA", "{}".format(v))
        self.cg_goto(cd)

    def cg_binop(self, op, node, dd, cd):
        self.alloc_zp()
        self.cg_form(node.cdr.cdr.car, DD_ZP, CD_NEXT)
        self.cg_form(node.cdr.car, DD_AC, CD_NEXT)
        op(dd, DD_AC, DD_ZP, cd)
        self.free_zp()

    def cg_ld16(self, dd, v):
        if dd == DD_AC:
            self.asm(None, "LDA", "#{}".format(v))
        elif dd == DD_XR:
            self.asm(None, "LDX", "#{}".format(v))
        elif dd == DD_YR:
            self.asm(None, "LDY", "#{}".format(v))
        else:
            raise ValueError("Unknown destination {}".format(dd))

    def cg_ld16_gv(self, dd, t):
        if dd == DD_AC:
            self.asm(None, "LDA", t)
        elif dd == DD_XR:
            self.asm(None, "LDX", t)
        elif dd == DD_YR:
            self.asm(None, "LDY", t)
        else:
            raise ValueError("Unknown destination {}".format(dd))

    def cg_call_libfn(self, fn_name, cd):
        if cd != CD_RET:
            self.asm(None, "JSR", fn_name)
            self.cg_goto(cd)
        else:
            self.asm(None, "JMP", fn_name)

    def cg_goto(self, cd):
        if isinstance(cd, tuple):
            true_branch = cd[0]
            false_branch = cd[1]

            if true_branch == CD_NEXT:
                if false_branch == CD_NEXT:
                    pass
                elif false_branch == CD_RET:
                    self.asm(None, "BNE", "*+3")
                    self.asm(None, "RTS", None)
                else:
                    self.asm(None, "BEQ", "L{}".format(false_branch))
            elif true_branch == CD_RET:
                if false_branch == CD_NEXT:
                    skippoint = self.make_label()
                    self.asm(None, "BEQ", "L{}".format(skippoint))
                    self.asm(None, "RTS", None)
                    self.cg_emit_label(skippoint)
                elif false_branch == CD_RET:
                    return self.cg_goto(CD_RET)
                else:
                    skippoint = self.make_label()
                    self.asm(None, "BEQ", "L{}".format(skippoint))
                    self.cg_goto(false_branch)
                    self.cg_emit_label(skippoint)

        elif cd == CD_NEXT:
            pass
        elif cd == CD_RET:
            self.asm(None, "RET", None)
        elif cd >= CD_LABEL:
            self.asm(None, "JMP", "L{}".format(cd))
        else:
            raise ValueError("Unknown control destination: {}".format(cd))

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
            DD_AC: 'A',
            DD_XR: 'X',
            DD_YR: 'Y',
        }[dd]


if __name__ == '__main__':
    Compiler().main(open(sys.argv[1]).read())

