"""SDK trust boundaries use source fixtures only; no SDK or compiler executes."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import native_dependencies as native
import native_sdk_proofs as sdk


class SourceSdkProofs(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.source=self.root/'source'
        self.source.mkdir()
        files=[]
        for name in sdk.FILES['focus-lightweight-view']:
            file=self.source/name
            file.write_text('authored inert source fixture '+name)
            files.append({'name':name,'sha256':native.sha256(file)})
        self.manifest=self.root/'corpus/native-sdk-sources.json'
        self.manifest.parent.mkdir()
        self.manifest.write_text(json.dumps({'format':1,'kind':'reviewed-source-sdk-recipes','recipes':[{
            'name':'focus-lightweight-view','repository':sdk.REVISIONS['focus-lightweight-view'][0],
            'revision':sdk.REVISIONS['focus-lightweight-view'][1],'files':files}]}))
        self.expected=hashlib.sha256(self.manifest.read_bytes()).hexdigest()

    def pin(self):
        with patch.object(sdk,'SOURCE_MANIFEST_SHA256',self.expected):
            return sdk.source_pin('focus-lightweight-view',self.source,self.root)

    def test_reviewed_file_fingerprints_are_required(self):
        self.assertEqual(len(self.pin().files),3)
        (self.source/'LightweightRenderingView.m').write_text('changed authored fixture')
        with self.assertRaisesRegex(ValueError,'immutable fingerprints'):
            self.pin()

    def test_changed_manifest_cannot_publish_a_new_approved_source(self):
        self.manifest.write_text(self.manifest.read_text()+' ')
        with self.assertRaisesRegex(ValueError,'manifest fingerprint changed'):
            self.pin()

    def test_source_alias_into_supplied_native_input_is_rejected_before_bytes(self):
        supplied.mkdir(parents=True)
        alias=self.root/'alias'
        alias.symlink_to(supplied,target_is_directory=True)
        with patch.object(native,'sha256',side_effect=AssertionError('must not read supplied bytes')):
            with self.assertRaisesRegex(ValueError,'protected input'):
                sdk.source_pin('focus-lightweight-view',alias,self.root)

    def test_transitive_headers_must_be_reviewed_source_or_installed_sdk(self):
        pin=self.pin()
        stray=self.root/'unreviewed.h'
        stray.write_text('authored stray header')
        with self.assertRaisesRegex(ValueError,'unreviewed transitive'):
            sdk.headers((stray,),pin,self.root/'oracle.mm',self.root)
        result=sdk.headers(tuple(Path(item.path) for item in pin.files),pin,self.root/'oracle.mm',self.root)
        self.assertEqual(len(result),3)

    def test_actual_mangled_symbol_names_are_preserved(self):
        mac='0000000000000010 T __ZN5ImGui10GetVersionEv\n0000000000000020 S _OBJC_CLASS_$_LightweightOpenGLView\n'
        self.assertEqual(sdk.symbol_names(mac,'arm64-apple-macosx27.0.0'),frozenset({
            '_ZN5ImGui10GetVersionEv','OBJC_CLASS_$_LightweightOpenGLView'}))
        linux='0000000000000010 T _ZN5ImGui10GetVersionEv\n'
        self.assertEqual(sdk.symbol_names(linux,'x86_64-unknown-linux-gnu'),frozenset({'_ZN5ImGui10GetVersionEv'}))

    def test_wrong_platform_recipe_does_not_reach_any_tool(self):
        with patch.object(sdk,'SOURCE_MANIFEST_SHA256',self.expected), \
             patch.object(native,'installed_tool',side_effect=AssertionError('must not resolve tools')):
            with self.assertRaisesRegex(ValueError,'requires macOS'):
                sdk.build('focus-lightweight-view',self.source,'x86_64-unknown-linux-gnu',
                    self.root/'artifacts/native-dependencies/fixture',Path('/usr/bin/clang++'),
                    Path('/usr/bin/ar'),Path('/usr/bin/nm'),self.root)

    def test_imgui_abi_requires_every_exact_layout_and_real_draw_counts(self):
        output = ('ImVec2 8 4 0 4\nImDrawVert 20 4 0 8 16\n'
                  'ImVectorPointer 16 8 0 4 8\nImGuiIO 14616 8\n'
                  'ImGuiStyle 1132 4\ncpu_draw_counts 12 18\n')
        abi = sdk.measured_abi('imgui-1.90.4-docking', output)
        self.assertEqual(abi.records[1], sdk.RecordAbi('ImDrawVert', 20, 4, (0, 8, 16)))
        self.assertEqual((abi.draw_vertices, abi.draw_indices), (12, 18))
        for malformed in (output + 'ImVec2 8 4 0 4\n', output.replace('ImGuiStyle 1132 4\n', ''),
                          output.replace('20 4 0 8 16', '24 4 0 8 16'),
                          output.replace('cpu_draw_counts 12 18', 'cpu_draw_counts 0 0'),
                          output.replace('cpu_draw_counts 12 18', 'cpu_draw_counts -1 6')):
            with self.subTest(malformed=malformed), self.assertRaises(ValueError):
                sdk.measured_abi('imgui-1.90.4-docking', malformed)

    def test_focus_abi_validates_object_selector_types_and_ivar_bounds(self):
        output = ('NSRect 32 8 0 16\nclass_storage 536 544 536\n'
                  'selectors object-getter object-setter void-swap\n')
        abi = sdk.measured_abi('focus-lightweight-view', output)
        self.assertEqual(abi.class_storage, sdk.ClassStorage(536, 544, 536))
        for malformed in (output.replace('536 544 536', '536 544 528'),
                          output.replace('536 544 536', '536 544 544'),
                          output.replace('object-setter', 'integer-setter'),
                          output + 'unknown 8 8\n'):
            with self.subTest(malformed=malformed), self.assertRaises(ValueError):
                sdk.measured_abi('focus-lightweight-view', malformed)


if __name__=='__main__':
    unittest.main()
