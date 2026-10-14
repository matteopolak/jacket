import tempfile
import unittest
from pathlib import Path
import sdl3_coverage as cov

class HeaderDiscovery(unittest.TestCase):
    def discover(self,text):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parents[1]/'artifacts/sdl3/tmp') as d:
            path=Path(d)/'SDL_gpu.h';path.write_text(text)
            return cov.discover(path)

    def test_record_callback_is_a_field_not_a_typedef(self):
        found=self.discover('typedef struct SDL_Test { bool (SDLCALL *close)(void *u); } SDL_Test;')
        self.assertIn(('typedef','SDL_Test'),[(i['kind'],i['name']) for i in found])
        self.assertNotIn(('callback','close'),[(i['kind'],i['name']) for i in found])

    def test_annotations_do_not_shadow_exported_procedure_names(self):
        found=self.discover('extern SDL_DECLSPEC SDL_ALLOC_SIZE2(1, 2) void * SDLCALL SDL_calloc(size_t n, size_t size);')
        self.assertEqual([i['name'] for i in found if i['kind']=='procedure'],['SDL_calloc'])

    def test_comments_macro_bodies_and_conditional_branches(self):
        found=self.discover('/* typedef int Fake; */\n#define SDL_TOKEN(T) \\\n typedef T bogus\n#ifdef SDL_PLATFORM_GDK\nextern SDL_DECLSPEC void SDLCALL SDL_GDKSuspendGPU(SDL_GPUDevice *device);\n#endif\n')
        self.assertNotIn('bogus',[i['name'] for i in found])
        function=next(i for i in found if i['name']=='SDL_GDKSuspendGPU')
        self.assertIn('ifdef SDL_PLATFORM_GDK',function['conditions'])
        macro=next(i for i in found if i['name']=='SDL_TOKEN')
        self.assertIn('typedef T bogus',macro['spelling'])

    def test_unknown_macro_cannot_be_classified_as_annotation(self):
        with self.assertRaisesRegex(ValueError,'Unclassified macro'):
            cov.macro_reason('SDL_NEW_API','SDL_NEW_API(x) ((x)+1)')

    def test_unexplained_gpu_api_is_fatal(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parents[1]/'artifacts/sdl3/tmp') as d:
            source=Path(d);header=source/'include/SDL3';header.mkdir(parents=True)
            (header/'SDL_gpu.h').write_text('extern SDL_DECLSPEC void SDLCALL SDL_NewGPUAPI(void);')
            reports={'linux-x64':{'excluded_functions':{},'wrapped_function_macros':[], 'excluded_object_macros':{},'unsupported_function_macros':[]}}
            with self.assertRaisesRegex(ValueError,'Unexplained installed-header declarations'):
                cov.reconcile(source,reports,{'linux-x64':{}})
            ledger=cov.reconcile(source,reports,{'linux-x64':{'SDL_NewGPUAPI':'linux-x64.jai::SDL_NewGPUAPI'}})
            self.assertEqual(ledger['unexplained'],[])

    def test_dependency_and_lifecycle_organization(self):
        self.assertLess(cov.ordering('SDL_GPUThing','record','SDL_gpu.h'),cov.ordering('SDL_CreateGPUThing','procedure','SDL_gpu.h'))
        self.assertLess(cov.ordering('SDL_CreateGPUThing','procedure','SDL_gpu.h'),cov.ordering('SDL_DestroyGPUThing','procedure','SDL_gpu.h'))

if __name__=='__main__':unittest.main()
