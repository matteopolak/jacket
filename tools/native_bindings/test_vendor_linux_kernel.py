import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/native_bindings'))
from vendor_linux_arm64 import folded_value


class KernelMacroTests(unittest.TestCase):
    def test_signed_folded_macro_precedes_unsigned_literal_operand(self):
        node = {'kind':'EnumConstantDecl','inner':[
            {'kind':'ConstantExpr','value':'-100','inner':[
                {'kind':'UnaryOperator','inner':[{'kind':'IntegerLiteral','value':'100'}]}]}]}
        self.assertEqual(folded_value(node),-100)

    def test_unresolved_macro_is_not_fabricated_as_zero(self):
        self.assertIsNone(folded_value({'kind':'EnumConstantDecl','inner':[
            {'kind':'DeclRefExpr'}]}))


if __name__=='__main__':
    unittest.main()
