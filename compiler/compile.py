#!/usr/bin/env python3


import sys

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

# Control Destinations
# RET implies a return after the current expression or statement.

CD_RET = 0
CD_NEXT = 1


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
            return int(t[1:], 8)
        elif '1' <= t[0] <= '9':
            return int(t, 10)
    else:
        raise ValueError("to_number called with empty token")


class Compiler:
    def __init__(self):
        self.parser_config = ParserConfig({}, dots_are_cons=True)
        self.assembly_listing = None

    def main(self, script=None):
        self.assembly_listing = []

        if script is None:
            script = "(* (/ (- 101 32) 180) 100)"

        print(script)
        print()

        tree = parse(script, self.parser_config)
        for node in tree:
            if is_pair(node):
                if node.car in ['-', '+', '/', '*']:
                    self.cg_expression(node, DD_HL, CD_RET)
                else:
                    raise ValueError("Unsupported: {}".format(node.car))
            else:
                if starts_with_decimal_digit(node):
                    self.cg_primary(node, DD_HL, CD_RET)
                else:
                    raise ValueError("Syntax error: {}".format(node))

        for line in self.assembly_listing:
            print(line)

    def cg_expression_lhs(self, node, dd, cd):
        if is_pair(node):
            self.cg_push_de()
            self.cg_expression(node, DD_HL, CD_NEXT)
            self.cg_pop_de()
            self.cg_goto(cd)
        else:
            self.cg_expression(node, DD_HL, cd)

    def cg_binop(self, op, node, dd, cd):
        if is_pair(node.cdr.car):
            self.cg_expression(node.cdr.cdr.car, DD_HL, CD_NEXT)
            self.cg_push_hl()
            self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
            self.cg_pop_de()
        else:
            self.cg_expression(node.cdr.cdr.car, DD_DE, CD_NEXT)
            self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
        op(dd, DD_HL, DD_DE, cd)

    def cg_expression(self, node, dd, cd):
        if is_pair(node):
            if node.car == '+':
                self.cg_binop(self.cg_add, node, dd, cd)
            elif node.car == '-':
                self.cg_binop(self.cg_subtract, node, dd, cd)
            elif node.car == '*':
                self.cg_binop(self.cg_multiply, node, dd, cd)
            elif node.car == '/':
                self.cg_binop(self.cg_divide, node, dd, cd)
            else:
                raise ValueError("Syntax error: {}".format(node.car))
        else:
            self.cg_primary(node, dd, cd)

    def cg_primary(self, t, dd, cd):
        n = to_number(t)

        if dd in [DD_BC, DD_DE, DD_HL]:
            self.cg_ld16(dd, n)
        else:
            raise ValueError("Unknown data destination: {}".format(dd))

        self.cg_goto(cd)

    def cg_add(self, dd, ds1, ds2, cd):
        def do_add(d, a, b):
            if (d == DD_HL) and (a == DD_HL):
                self.asm(None, "ADD", "HL,{}".format(self.to_reg(b)))
            else:
                self.asm(None, "LD", "A,{}".format(self.to_reg(a)[1]))
                self.asm(None, "ADD", "A,{}".format(self.to_reg(b)[1]))
                self.asm(None, "LD", "{},A".format(self.to_reg(d)[1]))
                self.asm(None, "LD", "A,{}".format(self.to_reg(a)[0]))
                self.asm(None, "ADC", "A,{}".format(self.to_reg(b)[0]))
                self.asm(None, "LD", "{},A".format(self.to_reg(d)[0]))

        if (dd == DD_HL) and (ds2 == DD_HL):
            do_add(dd, ds2, ds1)
        else:
            do_add(dd, ds1, ds2)
        self.cg_goto(cd)

    def cg_subtract(self, dd, ds1, ds2, cd):
        self.asm(None, "LD", "A,{}".format(self.to_reg(ds1)[1]))
        self.asm(None, "SUB", "A,{}".format(self.to_reg(ds2)[1]))
        self.asm(None, "LD", "{},A".format(self.to_reg(dd)[1]))
        self.asm(None, "LD", "A,{}".format(self.to_reg(ds1)[0]))
        self.asm(None, "SBC", "A,{}".format(self.to_reg(ds2)[0]))
        self.asm(None, "LD", "{},A".format(self.to_reg(dd)[0]))
        self.cg_goto(cd)

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
        if cd == CD_NEXT:
            pass
        elif cd == CD_RET:
            self.asm(None, "RET", None)
        else:
            raise ValueError("Unknown control destination: {}".format(cd))

    def cg_push_hl(self):
        self.asm(None, "PUSH", "HL")

    def cg_push_de(self):
        self.asm(None, "PUSH", "DE")

    def cg_pop_de(self):
        self.asm(None, "POP", "DE")

    def asm(self, label, mnem, oper):
        if label is not None:
            self.assembly_listing.append("{}:".format(label))
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
    Compiler().main(sys.argv[1])

