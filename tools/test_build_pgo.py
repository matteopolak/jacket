import json
import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build_pgo


class BuildEnvTests(unittest.TestCase):
    def test_extends_the_target_rustflags_ci_already_sets(self):
        env = build_pgo.build_env("aarch64-pc-windows-msvc",
                                  {"CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS": "-C target-feature=+crt-static"},
                                  "t", ["-Cprofile-generate=p"])
        self.assertEqual(env["CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS"],
                         "-C target-feature=+crt-static -Cprofile-generate=p")
        self.assertEqual(env["CARGO_TARGET_DIR"], "t")
        # MSVC's cl.exe gets no clang flags.
        self.assertFalse(any(k.startswith("CFLAGS") for k in env))

    def test_rustflags_overrides_target_flags_so_it_is_the_one_extended(self):
        env = build_pgo.build_env("x86_64-unknown-linux-gnu", {"RUSTFLAGS": "-Ctarget-cpu=native"}, "t",
                                  ["-Cprofile-use=a"])
        self.assertEqual(env["RUSTFLAGS"], "-Ctarget-cpu=native -Cprofile-use=a")
        self.assertNotIn("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", env)

    def test_c_code_is_not_instrumented(self):
        env = build_pgo.build_env("aarch64-apple-darwin", {"CFLAGS": "-O2"}, "t", ["-Cprofile-generate=p"])
        self.assertEqual(env["CFLAGS_aarch64_apple_darwin"], "-O2 -fno-profile-generate -fno-profile-use")


class LspSessionTests(unittest.TestCase):
    def test_session_is_framed_and_ends_with_exit(self):
        done = mock.Mock(returncode=0, stdout=b"")
        with mock.patch.object(build_pgo.subprocess, "run", return_value=done) as run:
            code, _ = build_pgo.lsp_session("jailsp", {})
        self.assertEqual(code, 0)
        data, messages = run.call_args.kwargs["input"], []
        while data:
            header, _, rest = data.partition(b"\r\n\r\n")
            n = int(header.split(b":")[1])
            messages.append(json.loads(rest[:n]))
            data = rest[n:]
        self.assertEqual(messages[0]["method"], "initialize")
        self.assertEqual([m["method"] for m in messages[-2:]], ["shutdown", "exit"])
        methods = {m["method"] for m in messages}
        for m in ("textDocument/didOpen", "textDocument/didChange", "textDocument/hover", "textDocument/completion"):
            self.assertIn(m, methods)


if __name__ == "__main__":
    unittest.main()
