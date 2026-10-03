"""Proof failures use inert authored fixtures; no native artifact executes."""
from dataclasses import asdict
from dataclasses import replace
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import native_dependencies as native
import native_dependency_proofs as proofs
from native_sdk_inventory import SdkRoot, inventory


LAYOUT = 'block 24 8 0 8 16\nallocation 32 8 0 8 16 24\ninfo 24 8 0 8 16\n'


class Proofs(unittest.TestCase):
    def reviewed_receipt(self):
        return native.BuildReceipt(native.VMA, 'arm64-apple-darwin',
                native.Fingerprint('/usr/bin/clang++', 'c' * 64),
                native.Fingerprint('/usr/bin/ar', 'd' * 64),
                (native.Fingerprint('/authored/vk_mem_alloc.h', native.VMA.source_sha256),
                 native.Fingerprint('/usr/include/vulkan/vulkan.h', 'e' * 64)),
                native.Fingerprint('/authored/fixture.a', 'f' * 64), '0' * 64,
                native.VMA_VIRTUAL_FLAGS)

    def test_fresh_rebuild_cannot_change_reviewed_transitive_inputs_before_execution(self):
        reviewed = self.reviewed_receipt()
        with patch.object(proofs, 'reviewed_inputs'):
            proofs.verify_rebuilt_inputs(reviewed, reviewed, Path('/authored'))
            for modified in (replace(reviewed, inputs=reviewed.inputs + (native.Fingerprint('/new/header.h', 'a' * 64),)),
                             replace(reviewed, inputs=reviewed.inputs[:-1]),
                             replace(reviewed, compiler=native.Fingerprint('/usr/bin/clang++', 'b' * 64))):
                with self.assertRaisesRegex(ValueError, 'differs from reviewed'):
                    proofs.verify_rebuilt_inputs(reviewed, modified, Path('/authored'))

    def test_sdk_headers_outside_installed_roots_do_not_reach_tool_or_runtime_execution(self):
        receipt = self.reviewed_receipt()
        receipt = replace(receipt, inputs=receipt.inputs + (native.Fingerprint('/authored/unreviewed.h', 'a' * 64),))
        with patch.object(native, 'installed_tool', side_effect=AssertionError('must not reach tools')):
            with self.assertRaisesRegex(ValueError, 'unreviewed SDK source root'):
                proofs.reviewed_inputs(receipt, Path('/authored/project'))

    def test_abi_measurement_requires_complete_unique_bounded_records(self):
        self.assertEqual(proofs.parse_abi_output(LAYOUT)[1].offsets, (0, 8, 16, 24))
        for invalid in (LAYOUT + 'info 24 8 0 8 16\n',
                        LAYOUT.replace('info 24 8 0 8 16\n', ''),
                        LAYOUT.replace('block 24 8', 'block 24 3'),
                        LAYOUT.replace('block 24 8', 'block 99999 8'),
                        LAYOUT.replace('info 24 8 0 8 16', 'info 24 8 0 16 8'),
                        LAYOUT.replace('info 24 8 0 8 16', 'info 24 8 0 8 24'),
                        LAYOUT.replace('block', 'unknown'),
                        LAYOUT.replace('24 8', '24 nope')):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                proofs.parse_abi_output(invalid)

    def test_saved_abi_evidence_cannot_grant_link_authority(self):
        proof = proofs.FreshSourceProof(proofs.RECIPE, 'arm64-apple-darwin',
                    native.Fingerprint('/authored/fixture.a', 'a' * 64),
                    proofs.parse_abi_output(LAYOUT), '/authored/receipt.json', 'b' * 64)
        self.assertFalse(asdict(proof)['link_authority'])
        rows = proofs.protocol(proof).splitlines()
        self.assertEqual(len(rows), 8)
        self.assertEqual(rows[0], 'JAI_NATIVE_SOURCE_ABI_V1')
        malicious = proofs.FreshSourceProof(proofs.RECIPE, 'target\ninjected',
                    proof.artifact, proof.records, proof.build_receipt, proof.probe_source_sha256)
        with self.assertRaisesRegex(ValueError, 'line breaks'):
            proofs.protocol(malicious)

    def test_protected_receipt_is_rejected_before_json_or_processes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            receipt = root / 'vendor/receipt.json'
            receipt.parent.mkdir()
            receipt.write_text('{}')
            with patch.object(proofs.json, 'loads', side_effect=AssertionError('must not read receipt')), \
                 patch.object(proofs.subprocess, 'run') as run:
                with self.assertRaisesRegex(ValueError, 'protected input'):
                    proofs.rebuild_and_prove(receipt, 'arm64-apple-darwin', root / 'output', root)
                run.assert_not_called()


class SdkConfiguration(unittest.TestCase):
    def test_configuration_is_named_absolute_and_unique(self):
        for invalid in ('vulkan=relative', 'vulkan', 'arbitrary=/tmp/sdk'):
            with self.assertRaises(ValueError):
                SdkRoot.parse(invalid)
        configured = SdkRoot.parse('vulkan=/tmp/authored-sdk')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            inventory((configured, configured), Path('/tmp/authored-project'))

    def test_sdk_symlink_into_original_inputs_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            supplied = root / 'corpus/upstream/sdk'
            supplied.mkdir(parents=True)
            alias = root / 'sdk-alias'
            alias.symlink_to(supplied, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'protected input'):
                inventory((SdkRoot('vulkan', alias),), root)

    def test_icd_manifest_presence_never_establishes_runtime_acceptance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sdk = root / 'sdk'
            directory = sdk / 'share/vulkan/icd.d'
            directory.mkdir(parents=True)
            (directory / 'authored.json').write_text(json.dumps({
                'ICD': {'api_version': '1.3.0', 'library_path': 'authored-driver-not-loaded'}}))
            result = inventory((SdkRoot('vulkan', sdk),), root)
            self.assertEqual(len(result['icds']), 1)
            self.assertFalse(result['icds'][0]['runtime_verified'])
            self.assertIsNone(result['icds'][0]['library_path_exists'])
            self.assertTrue(all(not project['native_acceptance'] for project in result['project_boundaries']))


if __name__ == '__main__':
    unittest.main()
