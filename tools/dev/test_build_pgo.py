import json
import unittest
from unittest import mock

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


    def test_pgo_is_skipped_only_on_arm64_windows(self):
        self.assertFalse(build_pgo.pgo_works("aarch64-pc-windows-msvc"))
        for target in ("x86_64-pc-windows-msvc", "aarch64-apple-darwin", "x86_64-unknown-linux-gnu"):
            self.assertTrue(build_pgo.pgo_works(target))


class LazyBackendTests(unittest.TestCase):
    def args(self, llvm):
        return mock.Mock(llvm=llvm, target="x86_64-unknown-linux-gnu", jobs=2)

    def test_a_lazy_build_also_builds_the_backend_library(self):
        with mock.patch.object(build_pgo, "run") as run:
            built = build_pgo.cargo_build(self.args("lazy"), "t", [])
        commands = [" ".join(map(str, call.args[0])) for call in run.call_args_list]
        self.assertTrue(any("-p jaic-cli --no-default-features --features lazy-llvm" in c for c in commands))
        self.assertTrue(any("-p jaic-backend --no-default-features --features static" in c for c in commands))
        self.assertEqual(sorted(built), ["backend", "jaic", "jailint", "jailsp"])
        self.assertEqual(built["backend"].name, build_pgo.BACKEND_FILE)

    def test_other_builds_have_no_backend_library(self):
        with mock.patch.object(build_pgo, "run") as run:
            built = build_pgo.cargo_build(self.args("static"), "t", [])
        self.assertNotIn("backend", built)
        self.assertFalse(any("jaic-backend" in " ".join(map(str, c.args[0])) for c in run.call_args_list))

    def test_the_library_keeps_its_own_file_name(self):
        self.assertEqual(build_pgo.file_name("backend"), build_pgo.BACKEND_FILE)
        self.assertEqual(build_pgo.file_name("jaic"), f"jaic{build_pgo.EXE}")


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
