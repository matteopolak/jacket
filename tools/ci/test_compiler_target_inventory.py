from functools import partial
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest
from unittest import mock

from compiler_target_inventory import PREFIX, collect, inventory, module_inventory, verify as verify_run
from toolkit.repo import ROOT


verify = partial(verify_run, expected_revision="a" * 40)

PAIRS = [["A", "linux-x64"], ["Extensions/B", "wasm"]]


def proof(assigned, completed=None):
    return [{"event": "full", "pairs": PAIRS}, {"event": "assigned", "pairs": assigned},
            *({"event": "completed", "pair": pair} for pair in (assigned if completed is None else completed)),
            {"event": "verified"}]


class ModuleProof(unittest.TestCase):
    def test_nextest_noise_is_not_retained(self):
        records = proof(PAIRS[:1])
        lines = ["compiler path /runner/private and arbitrary output\n"]
        lines += ["    " + PREFIX + json.dumps(record) + "\n" for record in records]
        self.assertEqual(collect(lines), records)
        self.assertNotIn("private", json.dumps(collect(lines)))

    def test_missing_duplicate_wrong_share_and_extra_fields_are_rejected(self):
        valid = proof(PAIRS[:1])
        invalid = [valid[:-1], [*valid[:-1], valid[-2], valid[-1]],
                   proof(PAIRS[:1], PAIRS[1:]),
                   [{**valid[0], "environment": {"TOKEN": "secret"}}, *valid[1:]],
                   [{**valid[0], "pairs": [PAIRS[0], PAIRS[0]]}, *valid[1:]]]
        for records in invalid:
            with self.subTest(records=records), self.assertRaises((AssertionError, ValueError)):
                collect([PREFIX + json.dumps(record) for record in records])

    def test_incomplete_valid_records_remain_available_for_diagnosis(self):
        values = proof(PAIRS[:1])[:-1]
        retained = []
        with self.assertRaises(AssertionError):
            collect([PREFIX + json.dumps(value) for value in values], emit=retained.append)
        self.assertEqual(retained, values)

    def test_invalid_record_does_not_close_the_producer_pipe_early(self):
        values = proof(PAIRS[:1])
        values.insert(1, {"event": "verified", "environment": {"TOKEN": "not retained"}})
        consumed, retained = [], []

        def lines():
            for index, value in enumerate(values):
                consumed.append(index)
                yield PREFIX + json.dumps(value)

        with self.assertRaises(ValueError):
            collect(lines(), emit=retained.append)
        self.assertEqual(consumed, list(range(len(values))))
        self.assertNotIn("not retained", json.dumps(retained))

    def test_truncated_json_and_no_records_are_rejected(self):
        for lines in [[PREFIX + '{"event":'], ["ordinary nextest output"]]:
            with self.assertRaises((AssertionError, ValueError)):
                collect(lines)


class FullProof(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.directories = []
        for index in range(2):
            directory = self.root / f"compiler-target-inventory-macos-15-intel-{index}"
            directory.mkdir()
            self.directories.append(directory)
            (directory / "metadata.json").write_text(json.dumps({"revision": "a" * 40,
                "partition": f"count:{index + 1}/2", "os": "Darwin", "arch": "x86_64", "rustc": "public pinned version"}))
            self.write_modules(directory, proof(PAIRS[index:index + 1]))
            for os_name in ("linux", "macos", "windows", "wasm"):
                cases = [{"type": "list", "name": f"{os_name}::{name}", "kind": "doc"} for name in ("one", "two")]
                assigned = cases[index:index + 1]
                outcomes = [{**case, "type": "test", "event": "passed"} for case in assigned]
                outcomes += [{"type": "run", "event": "finished", "failed": 0}]
                for suffix, records in [("all", cases), ("part", assigned), ("outcomes", outcomes)]:
                    self.write_lines(directory / f"doc-{os_name}-{suffix}.jsonl", records)
        rows = [{"runner": "macos-15-intel", "architecture": "x86_64", "parts": ",targets,", "shard": "1/1", "targets_part": f"{k}/2"} for k in (1, 2)]
        self.matrix = mock.patch("ci_changes.test_matrix", return_value=rows)

    def write_lines(self, path, records):
        path.write_text("".join(json.dumps(record) + "\n" for record in records))

    def write_modules(self, directory, records):
        self.write_lines(directory / "modules.jsonl", records)
        (directory / "module-status.json").write_text(json.dumps({"command": 0, "tee": 0, "capture": 0}))

    def test_actual_pairs_and_doc_outcomes_form_disjoint_complete_unions(self):
        with self.matrix:
            result = verify(self.root, True)
        self.assertEqual(result["macos-15-intel"]["module_pairs"], 2)
        self.assertEqual(result["macos-15-intel"]["docs"]["linux"], 2)

    def test_equal_count_full_replacement_and_execution_swap_fail(self):
        for replacement in [proof(PAIRS[1:]), proof(PAIRS[:1], PAIRS[1:])]:
            self.write_modules(self.directories[0], replacement)
            with self.assertRaises(AssertionError):
                module_inventory(self.directories)
        replacement = proof([["Different", "linux-x64"]])
        replacement[0] = {"event": "full", "pairs": [["Different", "linux-x64"], PAIRS[1]]}
        self.write_modules(self.directories[0], replacement)
        with self.assertRaisesRegex(AssertionError, "different full"):
            module_inventory(self.directories)

    def test_complete_cross_share_assignment_swap_is_rejected(self):
        for index, directory in enumerate(self.directories):
            self.write_modules(directory, proof(PAIRS[1-index:2-index]))
        with self.matrix, self.assertRaisesRegex(AssertionError, "assigned.*partition"):
            verify(self.root, True)

    def test_empty_module_and_doc_catalogues_are_rejected(self):
        for directory in self.directories:
            self.write_modules(directory, [{"event": "full", "pairs": []}, {"event": "assigned", "pairs": []}, {"event": "verified"}])
        with self.assertRaisesRegex(AssertionError, "empty"):
            module_inventory(self.directories)
        for index, directory in enumerate(self.directories):
            self.write_modules(directory, proof(PAIRS[index:index + 1]))
            self.write_lines(directory / "doc-linux-all.jsonl", [])
            self.write_lines(directory / "doc-linux-part.jsonl", [])
            self.write_lines(directory / "doc-linux-outcomes.jsonl", [{"type": "run", "event": "finished", "failed": 0}])
        with self.matrix, self.assertRaisesRegex(AssertionError, "empty"):
            verify(self.root, True)

    def test_unexplained_or_missing_dependency_doc_skips_are_rejected(self):
        path = self.directories[0] / "doc-linux-outcomes.jsonl"
        original = [json.loads(line) for line in path.read_text().splitlines()]
        for category in [None, "prerequisite", "cross-target-runner", "declared-or-runtime-skip"]:
            values = [{**original[0], "event": "skipped", "reason_category": category}, original[-1]]
            self.write_lines(path, values)
            with self.subTest(category=category), self.matrix, self.assertRaisesRegex(AssertionError, "unexpected doc skip"):
                verify(self.root, True)

    def test_declared_skips_remain_visible_and_flaky_doc_checks_fail(self):
        path = self.directories[0] / "doc-linux-outcomes.jsonl"
        original = [json.loads(line) for line in path.read_text().splitlines()]
        self.write_lines(path, [{**original[0], "event": "skipped", "reason_category": "declared-ignore"}, original[-1]])
        with self.matrix:
            self.assertEqual(verify(self.root, True)["macos-15-intel"]["docs"]["linux"], 2)
        self.write_lines(path, [{**original[0], "event": "flaky"}, {**original[-1], "flaky": 1}])
        with self.matrix, self.assertRaisesRegex(AssertionError, "flaky"):
            verify(self.root, True)

    def test_all_skipped_docs_cannot_prove_conformance(self):
        for directory in self.directories:
            path = directory / "doc-linux-outcomes.jsonl"
            values = [json.loads(line) for line in path.read_text().splitlines()]
            values[0].update(event="skipped", reason_category="declared-ignore")
            self.write_lines(path, values)
        with self.matrix, self.assertRaisesRegex(AssertionError, "no verified doc"):
            verify(self.root, True)

    def test_stale_source_cannot_prove_the_requested_head(self):
        with self.matrix, self.assertRaisesRegex(AssertionError, "stale source"):
            verify_run(self.root, True, expected_revision="b" * 40)

    def test_nonzero_command_or_missing_platform_cannot_prove_acceptance(self):
        (self.directories[0] / "module-status.json").write_text(json.dumps({"command": 100, "tee": 0, "capture": 0}))
        with self.assertRaisesRegex(AssertionError, "command"):
            module_inventory(self.directories)
        with self.assertRaisesRegex(AssertionError, "platform"):
            verify(self.root, True)


@unittest.skipUnless(os.name == "posix", "compiler target capture runs in Unix CI jobs")
class WorkflowCapture(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        (self.bin / "python3").symlink_to(sys.executable)
        self.env = {**os.environ, "PATH": str(self.bin) + os.pathsep + os.environ["PATH"],
                    "RUNNER_TEMP": str(self.root), "GITHUB_SHA": "a" * 40,
                    "INVENTORY": "true", "JACKET_STDLIB_TARGETS_PART": "1/2"}
        self.tool("rustc", 'print("rustc 1.99.0 (abcdef123 2026-01-01)")')
        body = "import json,os,sys\nfrom pathlib import Path\n"
        body += f"rows={proof(PAIRS[:1])!r}\n"
        body += 'Path(os.environ["RUNNER_TEMP"],"command.json").write_text(json.dumps({"args":sys.argv[1:],"success_output":os.environ.get("NEXTEST_SUCCESS_OUTPUT")}))\n'
        body += 'for row in rows:\n if not (os.environ.get("TRUNCATE") and row["event"]=="verified"):print("STDLIB_TARGET_INVENTORY "+json.dumps(row),file=sys.stderr)\n'
        body += 'raise SystemExit(int(os.environ.get("COMMAND_STATUS","0")))\n'
        self.tool("cargo", body)

    def tool(self, name, body):
        path = self.bin / name
        path.write_text(f"#!{sys.executable}\n" + body + "\n")
        path.chmod(0o755)
        return path

    def target_step(self, **extra):
        text = (ROOT / ".github/workflows/ci.yml").read_text()
        step = text.split("      - name: Stdlib type-checks and Objective-C selectors\n", 1)[1].split("      - name: Stdlib doc examples", 1)[0]
        shell = textwrap.dedent(step.split("        run: |\n", 1)[1])
        return subprocess.run(["bash", "-eo", "pipefail", "-c", shell], cwd=ROOT,
                              env={**self.env, **extra}, capture_output=True, text=True)

    def test_existing_argv_and_all_underlying_exit_codes_are_preserved(self):
        result = self.target_step()
        self.assertEqual(result.returncode, 0, result.stderr)
        command = json.loads((self.root / "command.json").read_text())
        self.assertEqual(command["args"], ["nextest", "run", "--profile", "ci", "--ignore-default-filter", "-p", "jaic-cli", "--locked", "--test", "stdlib_targets", "--test", "objc_selectors"])
        self.assertEqual(command["success_output"], "final")
        self.assertEqual(self.target_step(COMMAND_STATUS="100").returncode, 100)
        status = json.loads((self.root / "compiler-target-inventory/module-status.json").read_text())
        self.assertEqual(status, {"command": 100, "tee": 0, "capture": 0})

    def test_truncated_capture_fails_even_when_the_command_passes(self):
        result = self.target_step(TRUNCATE="true")
        self.assertNotEqual(result.returncode, 0)
        status = json.loads((self.root / "compiler-target-inventory/module-status.json").read_text())
        self.assertEqual(status["command"], 0)
        self.assertNotEqual(status["capture"], 0)
        self.assertTrue((self.root / "compiler-target-inventory/modules.jsonl").read_text())

    def test_second_share_omits_objc_and_default_does_not_collect(self):
        result = self.target_step(INVENTORY="false", JACKET_STDLIB_TARGETS_PART="2/2")
        self.assertEqual(result.returncode, 0, result.stderr)
        command = json.loads((self.root / "command.json").read_text())
        self.assertNotIn("objc_selectors", command["args"])
        self.assertNotEqual(command["success_output"], "final")
        self.assertFalse((self.root / "compiler-target-inventory").exists())


    def test_doc_capture_preserves_each_os_check_only_command_and_failure(self):
        body = r"""import json,os,sys
from pathlib import Path
args=sys.argv[1:]
assert args[:4]==["test","-q","--doc","--check-only"]
os_name=args[args.index("--os")+1]
with Path(os.environ["RUNNER_TEMP"],"doc-commands.jsonl").open("a") as out:out.write(json.dumps(args)+"\n")
cases=[{"type":"list","name":os_name+"::"+name,"kind":"doc"} for name in ["one","two"]]
if "--partition" in args:
 share=args[args.index("--partition")+1]
 assert share in ["count:1/2","count:2/2"]
 cases=cases[:1] if share=="count:1/2" else cases[1:]
failed=os.environ.get("DOC_FAIL")==os_name and "--list" not in args
if "--list" in args:
 for case in cases:print(json.dumps(case))
else:
 for case in cases:print(json.dumps({**case,"type":"test","event":"failed" if failed else "passed"}))
 print(json.dumps({"type":"run","event":"finished","failed":int(failed)}))
raise SystemExit(int(failed))
"""
        jacket = self.tool("jacket", body)
        directories = []
        for share in ["1/2", "2/2"]:
            directory = self.root / share.replace("/", "-")
            directories.append(directory)
            result = subprocess.run(["bash", "tools/ci/compiler_doc_inventory.sh", str(jacket), str(directory), share],
                                    cwd=ROOT, env=self.env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        for os_name in ("linux", "macos", "windows", "wasm"):
            whole, outcomes = inventory(directories, f"doc-{os_name}")
            self.assertEqual(len(whole), 2)
            self.assertEqual(len(outcomes), 2)
        failed = self.root / "failed-docs"
        result = subprocess.run(["bash", "tools/ci/compiler_doc_inventory.sh", str(jacket), str(failed), "1/2"],
                                cwd=ROOT, env={**self.env, "DOC_FAIL": "linux"}, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(all((failed / f"doc-{os_name}-outcomes.jsonl").exists() for os_name in ("linux", "macos", "windows", "wasm")))
