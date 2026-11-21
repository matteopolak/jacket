import unittest

import linux_cabi as cabi


def field(name, qual, desugared=None):
    ty = {'qualType': qual}
    if desugared:
        ty['desugaredQualType'] = desugared
    return {'kind': 'FieldDecl', 'name': name, 'type': ty}


def context(nodes, long_bytes=4, keep=()):
    return cabi.Context({'inner': nodes}, long_bytes, set(keep))


class FunctionTests(unittest.TestCase):
    def function(self, name, signature, params, symbol=None):
        return {'kind': 'FunctionDecl', 'name': name, 'mangledName': symbol or name,
                'type': {'qualType': signature},
                'inner': [{'kind': 'ParmVarDecl', 'name': n, 'type': {'qualType': t}} for n, t in params]}

    def test_long_follows_the_target_width(self):
        node = self.function('seekdir', 'void (DIR *, long)', [('dirp', 'DIR *'), ('pos', 'long')])
        for width, spelled in ((4, 's32'), (8, 's64')):
            ctx = context([node], width, keep=['DIR'])
            self.assertEqual(cabi.function_declaration(ctx, 'seekdir'),
                             (['dirp: *DIR', 'pos: ' + spelled], 'void', 'seekdir'))

    def test_the_link_symbol_is_the_headers_own(self):
        node = self.function('clock_gettime', 'int (clockid_t, struct timespec *)',
                             [('id', 'clockid_t'), ('tp', 'struct timespec *')], '__clock_gettime64')
        ctx = context([node], keep=['clockid_t', 'timespec'])
        self.assertEqual(cabi.function_declaration(ctx, 'clock_gettime')[2], '__clock_gettime64')

    def test_attributes_and_variadics_are_read_through(self):
        node = self.function('open', 'int (const char *, int, ...) __attribute__((nonnull (1)))',
                             [('file', 'const char *'), ('flags', 'int')])
        params, ret, _ = cabi.function_declaration(context([node]), 'open')
        self.assertEqual((params, ret), (['file: *u8', 'flags: s32', '__args: ..Any'], 's32'))


class RecordTests(unittest.TestCase):
    def record(self, name, fields, tag='struct'):
        return {'kind': 'RecordDecl', 'name': name, 'tagUsed': tag, 'completeDefinition': True,
                'inner': fields}

    def test_a_struct_uses_the_targets_widths(self):
        ctx = context([self.record('timespec', [field('tv_sec', '__time64_t', 'long long'),
                                                 field('tv_nsec', '__syscall_slong_t', 'long')])])
        self.assertEqual(cabi.shape(ctx, 'timespec'),
                         'timespec :: struct {\n    tv_sec: s64;\n    tv_nsec: s32;\n}')

    def test_a_union_overlays_its_first_member(self):
        ctx = context([self.record('sigval', [field('sival_int', 'int'), field('sival_ptr', 'void *')], 'union')])
        self.assertIn('#overlay (sival_int) sival_ptr: *void;', cabi.shape(ctx, 'sigval'))

    def test_unnamed_padding_bit_fields_become_bytes(self):
        pad = {'kind': 'FieldDecl', 'isBitfield': True, 'type': {'qualType': 'int'},
               'inner': [{'kind': 'ConstantExpr', 'value': '32'}]}
        ctx = context([self.record('t', [field('a', 'int'), pad])])
        self.assertIn('[4] u8', cabi.shape(ctx, 't'))

    def test_a_member_is_named_by_the_macro_that_stands_for_it(self):
        ctx = cabi.Context({'inner': [self.record('stat', [field('st_mtim', 'struct timespec')])]},
                           4, {'timespec'}, {'st_mtim': 'st_mtime'})
        self.assertIn('st_mtime: timespec;', cabi.shape(ctx, 'stat'))

    def test_a_named_bit_field_is_never_flattened(self):
        named = {'kind': 'FieldDecl', 'isBitfield': True, 'name': 'flag', 'type': {'qualType': 'int'},
                 'inner': [{'kind': 'ConstantExpr', 'value': '1'}]}
        ctx = context([self.record('t', [named])])
        with self.assertRaises(cabi.Unsupported):
            cabi.shape(ctx, 't')


if __name__ == '__main__':
    unittest.main()
