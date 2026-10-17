"""Independent metadata vectors; no target executable or native network run."""

import contextlib
import importlib.util
import io
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "burst_diagnostics", ROOT / "tools/check_io_burst_diagnostics.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# Public /proc/net/tcp columns: slot, endpoints, state, queues, timer, RTO
# count, uid, zero-window probes, inode. Deliberately omit optional tail fields.
HEADER = "sl local_address rem_address st tx_queue:rx_queue tr:tm->when retrnsmt uid timeout inode\n"
LISTENER = "0: 0100007F:1234 00000000:0000 0A 00000000:0000002A 00:00000000 00000000 1000 0 23\n"
CONNECTING = "1: 0100007F:4567 0100007F:1234 02 00000001:00000000 01:00000019 0000000A 1000 0 24\n"
FOREIGN = "2: 0100007F:9999 0100007F:8888 01 00000000:00000000 00:00000000 00000000 1000 0 99\n"


class MetadataTests(unittest.TestCase):
    def test_process_identity_separates_namespace_and_child(self):
        result = module.tcp_observations(
            HEADER + LISTENER + CONNECTING + FOREIGN, {23, 24}
        )
        self.assertEqual(result["namespace_states"], {"0A": 1, "02": 1, "01": 1})
        self.assertEqual(result["owned_states"], {"0A": 1, "02": 1})
        self.assertEqual(result["owned_listeners"][0]["rx_queue_raw"], 42)
        pending = result["owned_syn_sent_examples"][0]
        self.assertEqual(pending["inode"], 24)
        self.assertEqual(pending["timer_active"], 1)
        self.assertEqual(pending["timer_expires_jiffies_raw"], 25)
        self.assertEqual(result["owned_syn_sent_rto_total"], 10)

    def test_unknown_descriptor_identity_is_explicit(self):
        result = module.tcp_observations(HEADER + CONNECTING, None)
        self.assertFalse(result["owned_socket_identity_available"])
        self.assertEqual(result["namespace_states"], {"02": 1})
        self.assertEqual(result["owned_states"], {})

    def test_bounded_examples_do_not_truncate_aggregate(self):
        result = module.tcp_observations(HEADER + CONNECTING * 20, {24})
        self.assertEqual(result["owned_states"]["02"], 20)
        self.assertEqual(result["owned_syn_sent_rto_total"], 200)
        self.assertEqual(len(result["owned_syn_sent_examples"]), 16)
        self.assertTrue(result["syn_sent_examples_truncated"])

    def test_malformed_rows_do_not_discard_valid_evidence(self):
        result = module.tcp_observations(HEADER + "short\n" + LISTENER, {23})
        self.assertEqual(result["malformed_rows"], 1)
        self.assertEqual(result["owned_states"], {"0A": 1})

    def test_unavailable_table_is_preserved(self):
        unavailable = {"unavailable": "permission denied"}
        self.assertEqual(module.tcp_observations(unavailable, {23}), unavailable)

    def test_cgroup_parent_limits_are_observed(self):
        def read(path):
            if str(path) == "/proc/42/cgroup":
                return "0::/parent/child\n"
            if str(path).endswith("cpu.stat"):
                return "nr_throttled 5\nthrottled_usec 900\n"
            return "max 100000\n"

        with patch.object(module, "read_optional", side_effect=read):
            result = module.cgroup_observations(42)
        self.assertEqual(
            [row["path"] for row in result["v2_ancestors"]],
            ["/sys/fs/cgroup/parent/child", "/sys/fs/cgroup/parent", "/sys/fs/cgroup"],
        )
        self.assertIn(
            "throttled_usec 900", result["v2_ancestors"][1]["files"]["cpu.stat"]
        )

    def test_cgroup_path_escape_is_rejected_without_followup_reads(self):
        with patch.object(
            module, "read_optional", return_value="0::/../../outside\n"
        ) as read:
            result = module.cgroup_observations(42)
        self.assertIn("unrepresentable", result["unavailable"])
        read.assert_called_once_with("/proc/42/cgroup")

    def test_v1_membership_is_not_reported_as_zero_throttling(self):
        with patch.object(module, "read_optional", return_value="2:cpu:/jobs\n"):
            result = module.cgroup_observations(42)
        self.assertIn("v1 not sampled", result["unavailable"])

    def test_sample_preserves_descriptor_races(self):
        descriptors = [Path("/proc/42/fd/3"), Path("/proc/42/fd/4")]

        def readlink(path):
            if str(path).endswith("ns/net"):
                return "net:[1]"
            if str(path).endswith("fd/3"):
                return "socket:[23]"
            raise OSError("closed")

        with (
            patch.object(module.Path, "iterdir", return_value=iter(descriptors)),
            patch.object(module.os, "readlink", side_effect=readlink),
            patch.object(module, "read_optional", return_value=HEADER + LISTENER),
            patch.object(module, "cgroup_observations", return_value={}),
        ):
            result = module.sample(42)
        self.assertEqual(result["open_descriptors"], 2)
        self.assertEqual(result["descriptor_link_reads_unavailable"], 1)
        self.assertTrue(result["namespace_kernel_matches_child"])
        self.assertEqual(result["tcp"]["/proc/42/net/tcp"]["owned_states"], {"0A": 1})

    def test_sampling_cap_never_kills_or_retries_original_program(self):
        process = Mock(pid=42)
        process.poll.return_value = None
        process.wait.return_value = -6
        receipt = {"samples": []}
        with (
            patch.object(module.time, "monotonic", side_effect=[0, 181, 182]),
            patch.object(module.time, "sleep") as sleep,
            patch.object(module, "sample", return_value={"observed": True}),
        ):
            result = module.observe_process(process, receipt, Mock())
        self.assertEqual(result, -6)
        self.assertTrue(receipt["observation_window_exhausted"])
        process.kill.assert_not_called()
        process.wait.assert_called_once()
        sleep.assert_not_called()

    def test_runtime_collection_refuses_mac_before_execution(self):
        with (
            patch.object(module.platform, "system", return_value="Darwin"),
            patch.object(
                sys,
                "argv",
                ["collector", "--program", "/absent", "--report", "/absent"],
            ),
            patch.object(module.subprocess, "Popen") as popen,
            contextlib.redirect_stderr(io.StringIO()),
            self.assertRaises(SystemExit) as stopped,
        ):
            module.main()
        self.assertEqual(stopped.exception.code, 2)
        popen.assert_not_called()


if __name__ == "__main__":
    unittest.main()
