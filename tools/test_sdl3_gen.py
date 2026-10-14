import unittest
import sdl3_gen as gen

class Types(unittest.TestCase):
    def test_anonymous_record_metadata_is_independent_of_clang_diagnostic_spelling(self):
        source = gen.Path('/cache/SDL3')
        old = 'union (unnamed union at /cache/SDL3/include/SDL3/SDL_gamepad.h:271:5)'
        new = 'union (unnamed at /cache/SDL3/include/SDL3/SDL_gamepad.h:271:5)'
        expected = 'union (unnamed at SDL3/SDL_gamepad.h:271:5)'
        self.assertEqual(gen.stable_c_type(old, source), expected)
        self.assertEqual(gen.stable_c_type(new, source), expected)
        self.assertEqual(gen.stable_c_type('const SDL_Event *', source), 'const SDL_Event *')
        self.assertIsNone(gen.stable_c_type(None, source))

    def test_c_pointer_constness_and_arrays(self):
        self.assertEqual(gen.jai_type('const char *const *'), '**u8')
        self.assertEqual(gen.jai_type('Uint8[128]'), '[128] Uint8')
        self.assertEqual(gen.jai_type('float'), 'float32')
    def test_callbacks_use_c_convention(self):
        self.assertEqual(gen.jai_type('bool (*)(void *, SDL_Event *)'), '(arg0: *void, arg1: *SDL_Event) -> bool #c_call')
    def test_callback_parameter_names_survive_type_erasure(self):
        self.assertEqual(gen.named_callback('int (*)(void *)','typedef int (*SDL_Test)(void *userdata);','SDL_Test'), '(userdata: *void) -> s32 #c_call')
    def test_windows_cdecl_attribute_has_same_abi(self):
        self.assertEqual(gen.jai_type('int (*)(void *) __attribute__((cdecl))'), '(arg0: *void) -> s32 #c_call')
    def test_unknown_type_fails(self):
        with self.assertRaises(ValueError): gen.jai_type('unknown_t')
    def test_variadic_c_signature(self):
        self.assertEqual(gen.jai_type('int (*)(const char *, ...)'), '(arg0: *u8, args: ..Any) -> s32 #c_call')

if __name__ == '__main__': unittest.main()
