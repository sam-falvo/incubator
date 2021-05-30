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
        self.de_ptr = 0

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

    def cg_expression(self, node, dd, cd):
        if is_pair(node):
            if node.car == '+':
                self.cg_tmp_push()
                self.cg_expression(node.cdr.cdr.car, DD_DE, CD_NEXT)
                self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
                self.cg_add(DD_HL, DD_DE)
                self.cg_tmp_pop()
                self.cg_ld16_r16(dd, DD_HL)
                self.cg_goto(cd)
            elif node.car == '-':
                self.cg_tmp_push()
                self.cg_expression(node.cdr.cdr.car, DD_DE, CD_NEXT)
                self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
                self.cg_subtract(DD_HL, DD_DE)
                self.cg_tmp_pop()
                self.cg_ld16_r16(dd, DD_HL)
                self.cg_goto(cd)
            elif node.car == '*':
                self.cg_tmp_push()
                self.cg_expression(node.cdr.cdr.car, DD_DE, CD_NEXT)
                self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
                self.cg_multiply(DD_HL, DD_DE)
                self.cg_tmp_pop()
                self.cg_ld16_r16(dd, DD_HL)
                self.cg_goto(cd)
            elif node.car == '/':
                self.cg_tmp_push()
                self.cg_expression(node.cdr.cdr.car, DD_DE, CD_NEXT)
                self.cg_expression(node.cdr.car, DD_HL, CD_NEXT)
                self.cg_divide(DD_HL, DD_DE)
                self.cg_tmp_pop()
                self.cg_ld16_r16(dd, DD_HL)
                self.cg_goto(cd)
            else:
                raise ValueError("Syntax error: {}".format(node.car))
        else:
            self.cg_primary(node, dd, cd)

    def cg_primary(self, t, dd, cd, negate=False):
        n = to_number(t)

        if negate:
            n = -n

        if dd in [DD_BC, DD_DE, DD_HL]:
            self.cg_ld16(dd, n)
        else:
            raise ValueError("Unknown data destination: {}".format(dd))

        self.cg_goto(cd)

    def cg_add(self, dd, ds):
        if ds == DD_HL:
            (ds, dd) = (dd, ds)

        if dd == DD_HL:
            self.asm(None, "ADD", "{},{}".format(self.to_reg(dd), self.to_reg(ds)))
        else:
            self.cg_op_pair("ADD", "ADC", dd, ds)

    def cg_subtract(self, dd, ds):
        self.cg_op_pair("SUB", "SBC", dd, ds)

    def cg_divide(self, dd, ds):
        self.asm(None, "CALL", "divide_{}_{}".format(self.to_reg(dd), self.to_reg(ds)))

    def cg_multiply(self, dd, ds):
        self.asm(None, "CALL", "multiply_{}_{}".format(self.to_reg(dd), self.to_reg(ds)))

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

    def cg_tmp_push(self):
        self.asm(None, "LD", "(TMPDE{}),DE".format(self.de_ptr))
        self.de_ptr = self.de_ptr + 1

    def cg_tmp_pop(self):
        self.de_ptr = self.de_ptr - 1
        self.asm(None, "LD", "DE,(TMPDE{})".format(self.de_ptr))

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

