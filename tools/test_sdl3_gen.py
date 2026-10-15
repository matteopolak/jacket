import unittest
import hashlib
import json
from pathlib import Path
import sdl3_gen as gen

class Types(unittest.TestCase):
    def test_anonymous_record_paths_normalize_native_and_clang_separators(self):
        source = gen.Path('cache/SDL3')
        location = (source.resolve() / 'include/SDL3/SDL_gamepad.h').as_posix()
        expected = 'union (unnamed at SDL3/SDL_gamepad.h:271:5)'
        for spelling in [location, location.replace('/', '\\')]:
            self.assertEqual(gen.stable_c_type(f'union (unnamed at {spelling}:271:5)', source), expected)

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
    def test_contract_anonymous_record_spelling_is_clang_version_independent(self):
        source=Path('artifacts/sdl3/SDL3-3.4.18')
        for kind in ('struct','union'):
            for anonymity in ('unnamed','anonymous'):
                expected=f'{kind} ({anonymity} at SDL3/SDL_gamepad.h:271:5)'
                for spelling in (f'{kind} ({anonymity} at ',f'{kind} ({anonymity} {kind} at '):
                    for prefix in ((source/'include').as_posix(),(source.resolve()/'include').as_posix()):
                        for location in (prefix+'/SDL3/SDL_gamepad.h:271:5)',(prefix+'/SDL3/SDL_gamepad.h:271:5)').replace('/','\\')):
                            self.assertEqual(gen.stable_c_type(spelling+location,source),expected)
        self.assertEqual(gen.stable_c_type('const SDL_GPUDevice *',source),'const SDL_GPUDevice *')
        self.assertIsNone(gen.stable_c_type(None,source))

    def test_windows_extraction_uses_msvc_builtin_availability(self):
        for plat in gen.TARGETS:
            flags=gen.header_profile_flags(plat)
            if plat.startswith('windows'):
                self.assertIn('-DSDL_HAS_BUILTIN(x)=0',flags)
                self.assertIn('-U__clang__',flags)
                self.assertIn('-U__GNUC__',flags)
            else:
                self.assertEqual(flags,[])

    def test_generated_windows_wrappers_do_not_call_absent_builtin_helpers(self):
        # These compiler-conditional helpers are absent in the pinned MSVC header
        # profile. The documented portable helpers and all desktop GPU APIs remain.
        for cpu in ('x64','arm64'):
            wrapper=(gen.ROOT/f'tools/native/sdl3/inline-windows-{cpu}.c').read_text()
            jai=(gen.OUT/f'windows-{cpu}.jai').read_text()
            for operation in ('mul','add'):
                name=f'SDL_size_{operation}_check_overflow'
                self.assertIn('JAI_SDL3_'+name+'(',wrapper)
                self.assertIn(name+' :: ',jai)
                self.assertNotIn(name+'_builtin',wrapper)
                self.assertNotIn(name+'_builtin',jai)
            self.assertIn('SDL_CreateGPUDevice :: ',jai)

    def test_checked_in_contract_hashes_match_canonical_payloads(self):
        inventory=json.loads((gen.OUT/'inventory.json').read_text())
        for plat in gen.TARGETS:
            report=inventory[plat]
            self.assertEqual(report['header_profile']['feature_flags'],gen.header_profile_flags(plat))
            self.assertEqual(report['contracts_sha256'],hashlib.sha256((gen.OUT/report['contracts']).read_bytes()).hexdigest())

    def test_all_desktop_gpu_procedures_remain_in_every_profile(self):
        coverage=json.loads((gen.OUT/'coverage.json').read_text())
        inventory=json.loads((gen.OUT/'inventory.json').read_text())
        gpu={item['name'] for item in coverage['headers']['SDL_gpu.h']['declarations']
             if item['kind']=='procedure' and item['name'] not in {'SDL_GDKSuspendGPU','SDL_GDKResumeGPU'}}
        self.assertEqual(len(gpu),95) # Pinned 3.4.18: 95 desktop + two GDK-only procedures.
        for plat,report in inventory.items():
            self.assertTrue(gpu<=set(report['functions']),f'{plat}: {sorted(gpu-set(report["functions"]))}')
        self.assertEqual(coverage['unexplained'],[])

if __name__ == '__main__': unittest.main()
