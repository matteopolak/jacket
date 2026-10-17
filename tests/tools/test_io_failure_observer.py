"""Numeric vectors and mocked supervisor controls; no network/native execution."""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
assigned_scratch = importlib.import_module("ci_test_scratch").assigned_scratch

spec = importlib.util.spec_from_file_location(
    "failure_observer", ROOT / "tools/io_failure_observer.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

HEADER = "sl local_address rem_address st tx_queue:rx_queue tr:tm->when retrnsmt uid timeout inode\n"
LISTEN = "0: 0100007F:1234 00000000:0000 0A 00000000:0000002A 00:00000000 00000000 1000 0 23\n"
SYN = "1: 0100007F:4567 0100007F:1234 02 00000001:00000000 01:00000019 00000009 1000 0 24\n"
DIAGNOSTIC = (
    "Io burst failure: backend=epoll round=12 elapsed_ms=135248 accepted_total=3569 handlers_finished_total=3569\n"
    "Io burst client: connect=31 first=TIMED_OUT code=110 write=0 first=NONE code=0 read=0 first=NONE code=0\n"
    "Io burst server: accept=0 first=NONE code=0 read=0 first=NONE code=0 write=0 first=NONE code=0\n"
)


class FailureObserverTests(unittest.TestCase):
    def test_source_digest_is_deterministic_and_detects_real_input_changes(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            root = Path(scratch)
            (root / "stdlib").mkdir()
            (root / "tools").mkdir()
            source = root / "stdlib/input.jai"
            source.write_text("// digest-only unit input\n")
            for name in ("io_stress.py", "io_failure_observer.py"):
                (root / "tools" / name).write_text("# digest-only unit input\n")
            compiler = root / "compiler-digest-input"
            compiler.write_text("digest-only bytes; never executed")
            before = module.source_identity(root, compiler)
            self.assertEqual(before, module.source_identity(root, compiler))
            self.assertEqual(before["source_files"], 3)
            (root / "stdlib/._metadata.jai").write_text("digest-only metadata sidecar")
            self.assertEqual(before, module.source_identity(root, compiler))
            source.write_text("// changed digest-only unit input\n")
            after = module.source_identity(root, compiler)
            self.assertNotEqual(before["source_sha256"], after["source_sha256"])
            self.assertEqual(before["compiler_sha256"], after["compiler_sha256"])

    def test_one_table_preserves_owned_and_namespace_domains_without_endpoints(self):
        result = module.tcp_metrics(
            HEADER + LISTEN + SYN + "malformed\n", {0: {23}, 6: {24}, 9: set()}
        )
        self.assertEqual(result["namespace_states"], {"0A": 1, "02": 1})
        self.assertEqual(result["owned"][0]["listener_rx"], [42])
        self.assertEqual(result["owned"][6]["syn_rto"], 9)
        self.assertEqual(result["owned"][9]["states"], {})
        self.assertEqual(result["malformed_rows"], 1)
        self.assertNotIn("0100007F", json.dumps(result))
        self.assertNotIn("inode", json.dumps(result))

    def test_unavailable_and_truncated_tables_remain_explicit(self):
        self.assertFalse(module.tcp_metrics(None, {1: {24}})["available"])
        with patch.object(module, "MAX_TCP_ROWS", 1):
            result = module.tcp_metrics(HEADER + LISTEN + SYN, {1: {24}})
        self.assertTrue(result["truncated"])
        self.assertEqual(result["owned"][1]["states"], {})

    def test_namespace_drop_metrics_do_not_include_unknown_text(self):
        def read(path):
            if path == "/proc/net/stat/nf_conntrack":
                return "entries insert_failed drop early_drop private\n10 02 03 04 aa\n10 05 06 07 bb\n"
            if path == "/proc/net/softnet_stat":
                return "00 03 02\n00 04 01\n"
            return None

        with patch.object(module, "read", side_effect=read):
            result = module.namespace_metrics()
        self.assertEqual(
            result["conntrack_stat"]["counters"],
            {"insert_failed": 7, "drop": 9, "early_drop": 11},
        )
        self.assertEqual(result["softnet"]["dropped"], 7)
        self.assertIsNone(result["sysctls"]["tcp_syn_retries"])
        self.assertNotIn("private", json.dumps(result))

    def test_report_selects_a_real_failure_and_passed_control_with_bounded_history(
        self,
    ):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            group = module.FailureGroup(Path(scratch), {"source_sha256": "test-digest"})
            try:
                for copy, status in ((0, 0), (6, 1), (11, 2)):
                    group.records[copy] = {
                        "exit": status,
                        "exit_phase": "runtime_exit",
                        "samples": module.deque(range(100), maxlen=module.HISTORY),
                    }
                group.finish([6, 11])
                receipt = json.loads((Path(scratch) / "receipt.json").read_text())
                copies = receipt["groups"][0]["copies"]
                self.assertEqual(
                    [(row["copy"], row["role"], row["exit"]) for row in copies],
                    [(6, "failed", 1), (0, "passed_control", 0)],
                )
                self.assertEqual(len(copies[0]["samples"]), module.HISTORY)
                self.assertEqual(receipt["groups"][0]["failed_copies"], [6, 11])
                with patch.object(module, "MAX_GROUPS", 1):
                    group.finish([6, 11])
                receipt = json.loads((Path(scratch) / "receipt.json").read_text())
                self.assertEqual(len(receipt["groups"]), 1)
                self.assertEqual(receipt["unretained_failed_groups"], 1)
            finally:
                group.finish([])

    def test_only_reviewed_numeric_burst_fields_are_retained(self):
        result = module.burst_numbers("unrelated private test marker\n" + DIAGNOSTIC)
        self.assertEqual(result["client"]["connect"], {"count": 31, "code": 110})
        self.assertEqual(result["accepted"], 3569)
        self.assertNotIn("private", json.dumps(result))
        self.assertEqual(module.burst_numbers("arbitrary stdout/error"), {})

    def test_supervisor_failure_is_one_original_launch_with_unchanged_timeout(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            group = module.FailureGroup(
                Path(scratch), {"program_sha256": "test-digest"}
            )
            child = Mock(pid=42, returncode=1)
            child.communicate.return_value = ("original output", DIAGNOSTIC)
            context = Mock()
            context.__enter__ = Mock(return_value=child)
            context.__exit__ = Mock(return_value=False)
            try:
                with patch.object(
                    module.subprocess, "Popen", return_value=context
                ) as launch:
                    result = group.run(6, Path("original-program"), Path(scratch), 180)
                self.assertEqual(result.returncode, 1)
                self.assertEqual(result.stdout, "original output")
                child.communicate.assert_called_once_with(timeout=180)
                child.kill.assert_not_called()
                launch.assert_called_once()
                group.finish([6])
                receipt = json.loads((Path(scratch) / "receipt.json").read_text())
                row = receipt["groups"][0]["copies"][0]
                self.assertEqual(row["copy"], 6)
                self.assertEqual(row["exit_phase"], "runtime_exit")
                self.assertEqual(row["burst"]["client"]["connect"]["code"], 110)
                self.assertNotIn("original output", json.dumps(receipt))
            finally:
                group.finish([])

    def test_only_original_supervisor_timeout_kills_and_reaps_once(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            group = module.FailureGroup(Path(scratch), {})
            child = Mock(pid=42, returncode=-9)
            child.communicate.side_effect = [
                subprocess.TimeoutExpired("original", 180),
                ("out", "err"),
            ]
            context = Mock()
            context.__enter__ = Mock(return_value=child)
            context.__exit__ = Mock(return_value=False)
            try:
                with patch.object(
                    module.subprocess, "Popen", return_value=context
                ) as launch:
                    with self.assertRaises(subprocess.TimeoutExpired):
                        group.run(7, Path("original"), Path(scratch), 180)
                child.kill.assert_called_once_with()
                self.assertEqual(child.communicate.call_count, 2)
                launch.assert_called_once()
                self.assertEqual(group.records[7]["exit_phase"], "supervisor_timeout")
            finally:
                group.finish([])

    def test_launch_error_retains_numeric_phase_without_replacement_child(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            group = module.FailureGroup(Path(scratch), {})
            try:
                with patch.object(
                    module.subprocess,
                    "Popen",
                    side_effect=OSError(11, "private test marker"),
                ) as launch:
                    with self.assertRaises(OSError):
                        group.run(6, Path("original"), Path(scratch), 180)
                launch.assert_called_once()
                group.finish([6])
                receipt = json.loads((Path(scratch) / "receipt.json").read_text())
                row = receipt["groups"][0]["copies"][0]
                self.assertEqual(row["exit_phase"], "launch_error")
                self.assertEqual(row["launch_errno"], 11)
                self.assertNotIn("private", json.dumps(receipt))
            finally:
                group.finish([])

    def test_shared_sampler_reads_two_tables_once_for_twelve_children(self):
        with tempfile.TemporaryDirectory(dir=assigned_scratch()) as scratch:
            group = module.FailureGroup(Path(scratch), {})
            try:
                for copy in range(12):
                    group.records[copy] = {
                        "pid": 100 + copy,
                        "exit_phase": "running",
                        "samples": module.deque(maxlen=module.HISTORY),
                    }
                with (
                    patch.object(
                        module,
                        "process_metrics",
                        return_value=(
                            {"available": True, "namespace_matches_observer": True},
                            {24},
                        ),
                    ),
                    patch.object(module, "namespace_metrics", return_value={}),
                    patch.object(module, "read", return_value=HEADER + SYN) as read,
                ):
                    group.sample()
                self.assertEqual(read.call_count, 2)
                for record in group.records.values():
                    self.assertEqual(
                        record["samples"][0]["tcp"][0]["owned"]["syn_rto"], 9
                    )
            finally:
                group.finish([])

    def test_all_thread_sum_is_bounded_and_races_not_fabricated_zero(self):
        self.assertEqual(
            module.cpu_limit("max 100000\n"),
            {
                "available": True,
                "unlimited": True,
                "quota_us": None,
                "period_us": 100000,
            },
        )
        self.assertEqual(module.cpu_limit("50000 100000\n")["quota_us"], 50000)
        self.assertEqual(module.cpu_limit(None), {"available": False})

        def read(path):
            if str(path).endswith("/1/schedstat"):
                return "100 200 3\n"
            if str(path).endswith("/2/schedstat"):
                return None
            return ""

        with (
            patch.object(
                module.Path,
                "iterdir",
                side_effect=[
                    iter([]),
                    iter(
                        [
                            Path("/proc/42/task/1"),
                            Path("/proc/42/task/2"),
                            Path("/proc/42/task/3"),
                        ]
                    ),
                ],
            ),
            patch.object(module, "MAX_TASKS", 2),
            patch.object(module.os, "readlink", return_value="same-namespace"),
            patch.object(module, "read", side_effect=read),
            patch.object(module, "cgroup_observations", return_value={}),
        ):
            metrics, _ = module.process_metrics(42)
        self.assertEqual(metrics["thread_schedstat"]["sum"], [100, 200, 3])
        self.assertEqual(metrics["thread_schedstat"]["unavailable"], 1)
        self.assertTrue(metrics["thread_schedstat"]["truncated"])
        self.assertFalse(metrics["cgroup_available"])


if __name__ == "__main__":
    unittest.main()
