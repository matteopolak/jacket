import hashlib
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from check_corpus import Evidence, Result, Source, Stage, Status
from check_native_examples import NativeCase, RuntimeExpectation, audit_calls, check_case

class NativeExamplesTests(unittest.TestCase):
    def fixture(self, root):
        path = root/'main.jai'; path.write_text('main :: () {}')
        source = Source('source:main.jai',path,hashlib.sha256(path.read_bytes()).hexdigest(),'', 'source',None)
        preload_path = root/'module.jai'; preload_path.write_text('Type :: #compiler_type;')
        preload = Source('reference:modules/module.jai',preload_path,hashlib.sha256(preload_path.read_bytes()).hexdigest(),'', 'reference',None)
        return NativeCase(source,'Reviewed pure standalone source',RuntimeExpectation(0,'',''),preload)

    def result(self, case):
        stages = {stage.value: Evidence(Status.NOT_RUN) for stage in Stage}
        stages['codegen'] = Evidence(Status.PASSED)
        return Result(case.source.id,str(case.source.path),case.source.sha256,'program','host',[],stages)

    def test_ir_requires_defined_main_and_direct_local_or_intrinsic_calls(self):
        self.assertEqual(audit_calls('define void @body() { ret void }\ndefine i32 @main() {\ncall void @body()\nret i32 0\n}'), ['body'])
        self.assertEqual(audit_calls('define i32 @main() {\ncall void @llvm.trap()\nret i32 0\n}'), ['llvm.trap'])
        for text in ['define i32 @main() {\ncall void @external()\n}', 'define i32 @main() {\ncall void %pointer()\n}', 'define void @other() { ret void }']:
            with self.assertRaises(ValueError): audit_calls(text)

    def test_external_ir_blocks_native_build(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);case=self.fixture(root);output=root/'output'
            output.write_text('define i32 @main() {\ncall void @external()\n}')
            with patch('check_native_examples.evaluate',return_value=self.result(case)), patch('check_native_examples.execute') as build:
                result,audit=check_case(case,root/'jai-rs',1,output)
            build.assert_not_called()
            self.assertEqual(result.stages['build'].status,Status.BLOCKED)
            self.assertEqual(audit['status'],'blocked')

    def test_runtime_uses_exact_build_fingerprint_and_actual_preload_profile(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);case=self.fixture(root);output=root/'output'
            output.write_text('define i32 @main() { ret i32 0 }')
            sha='a'*64
            with patch('check_native_examples.evaluate',return_value=self.result(case)) as frontend, patch('check_native_examples.execute',return_value=Evidence(Status.PASSED,output_sha256=sha)) as build, patch('check_native_examples.run_native',return_value=Evidence(Status.PASSED,output_sha256=sha)) as runtime:
                result,audit=check_case(case,root/'jai-rs',1,output)
            self.assertEqual(frontend.call_args.kwargs['bootstrap'],'search')
            self.assertEqual(build.call_args.kwargs['bootstrap'],'search')
            self.assertEqual(runtime.call_args.kwargs['expected_sha256'],sha)
            self.assertEqual(result.stages['run'].output_sha256,sha)
            self.assertEqual(audit['status'],'passed')

    def test_source_drift_after_ir_review_blocks_native_build(self):
        with TemporaryDirectory() as directory:
            root=Path(directory);case=self.fixture(root);output=root/'output'
            output.write_text('define i32 @main() { ret i32 0 }')
            case.source.path.write_text('changed source')
            with patch('check_native_examples.evaluate',return_value=self.result(case)), patch('check_native_examples.execute') as build:
                result,_=check_case(case,root/'jai-rs',1,output)
            build.assert_not_called()
            self.assertEqual(result.stages['build'].status,Status.BLOCKED)

if __name__ == '__main__':
    unittest.main()
