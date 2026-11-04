import json
import subprocess
import tempfile
from pathlib import Path
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


class TrainingEvidenceTests(unittest.TestCase):
    def test_new_invocation_removes_stale_training_evidence_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp)
            (work / "training" / "pgo").mkdir(parents=True)
            (work / "training" / "pgo" / "summary.json").write_text("old")
            (work / "merged.profdata").write_text("profile")
            build_pgo.reset_training_evidence(work)
            self.assertFalse((work / "training").exists())
            self.assertEqual((work / "merged.profdata").read_text(), "profile")
            build_pgo.reset_training_evidence(work)

    def test_failed_step_retains_output_and_bounded_diagnostic(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "step.log"
            def fail(cmd, **kwargs):
                kwargs["stdout"].write(b"x" * 20000 + b"\nmissing backend library\n")
                return mock.Mock(returncode=1)
            with mock.patch.object(build_pgo, "run", side_effect=fail), mock.patch.object(build_pgo, "log") as log:
                result = build_pgo.training_step("native -O0", ["jaic", "-O0"], {}, path, 30)
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["exit_code"], 1)
            self.assertEqual(result["command"], ["jaic", "-O0"])
            self.assertIn("missing backend library", path.read_text())
            diagnostics = "\n".join(str(c.args[0]) for c in log.call_args_list)
            self.assertIn("missing backend library", diagnostics)
            self.assertLess(len(diagnostics), 17000)

    def test_expected_lint_diagnostics_are_recorded_without_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            with mock.patch.object(build_pgo, "run", return_value=mock.Mock(returncode=1)):
                result = build_pgo.training_step("lint", ["jailint"], {}, Path(tmp) / "lint.log", 30, expected=(0, 1))
        self.assertEqual(result["status"], "completed")
        self.assertEqual(result["exit_code"], 1)

    def test_timeout_keeps_partial_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "timeout.log"
            def timeout(cmd, **kwargs):
                kwargs["stdout"].write(b"partial output\n")
                raise subprocess.TimeoutExpired(cmd, 30)
            with mock.patch.object(build_pgo, "run", side_effect=timeout):
                result = build_pgo.training_step("native", ["jaic"], {}, path, 30)
            self.assertEqual(result["exit_code"], "timeout")
            self.assertEqual(result["status"], "failed")
            self.assertEqual(path.read_text(), "partial output\n")

    def test_inventory_distinguishes_requested_modes_and_writes_summary(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            def step(name, cmd, env, path, timeout, **kwargs):
                return {"name": name, "command": list(map(str, cmd)), "exit_code": 0, "status": "completed"}
            with mock.patch.object(build_pgo, "training_step", side_effect=step), mock.patch.object(build_pgo, "lsp_session", return_value=(0, "session")):
                problems = build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, True, Path(tmp))
            report = json.loads((Path(tmp) / "summary.json").read_text())
        self.assertEqual(problems, [])
        commands = {r["name"]: r["command"] for r in report["steps"]}
        self.assertNotIn("--opt", commands["sweep (check and run)"])
        for opt in ("O0", "O2"):
            cmd = commands[f"sweep (native -{opt})"]
            self.assertEqual(cmd[cmd.index("--opt") + 1], opt)
        self.assertEqual(report["problems"], [])

    def test_real_failed_command_keeps_merged_stdout_and_stderr(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "failed.log"
            result = build_pgo.training_step("failure", [build_pgo.sys.executable, "-c",
                "import sys; print('native request'); print('backend unavailable', file=sys.stderr); sys.exit(7)"],
                None, output, 30)
            text = output.read_text()
        self.assertEqual(result["exit_code"], 7)
        self.assertEqual(result["status"], "failed")
        self.assertIn("native request", text)
        self.assertIn("backend unavailable", text)

    def test_failed_mode_is_not_reported_as_training_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            def step(name, cmd, env, path, timeout, **kwargs):
                failed = name == "sweep (native -O2)"
                return {"name": name, "command": list(map(str, cmd)), "exit_code": 1 if failed else 0,
                        "status": "failed" if failed else "completed"}
            with mock.patch.object(build_pgo, "training_step", side_effect=step), mock.patch.object(build_pgo, "lsp_session", return_value=(0, "session")):
                problems = build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, True, Path(tmp))
            report = json.loads((Path(tmp) / "summary.json").read_text())
        self.assertEqual(problems, ["sweep (native -O2) (exit 1)"])
        self.assertEqual(report["problems"], problems)

    def test_launch_abort_preserves_inventory_and_propagates(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            with mock.patch.object(build_pgo, "training_step", side_effect=FileNotFoundError("missing jaic")):
                with self.assertRaises(FileNotFoundError):
                    build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, True, Path(tmp))
            report = json.loads((Path(tmp) / "summary.json").read_text())
        self.assertEqual(report["steps"][0]["status"], "aborted")
        self.assertIn("missing jaic", report["steps"][0]["error"])
        self.assertTrue(all(r["status"] == "not_started" for r in report["steps"][1:]))

    def test_lsp_failure_prints_retained_diagnostic(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            def lsp(binary, env, output):
                output.write_text("LSP protocol failed")
                return 1, "session"
            with mock.patch.object(build_pgo, "training_step", return_value={"status": "completed"}), mock.patch.object(build_pgo, "lsp_session", side_effect=lsp), mock.patch.object(build_pgo, "log") as log:
                problems = build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, False, Path(tmp))
        self.assertEqual(problems, ["jailsp session (exit 1)"])
        self.assertTrue(any("LSP protocol failed" in str(c.args[0]) for c in log.call_args_list))

    def test_without_backend_inventory_has_no_native_steps(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            with mock.patch.object(build_pgo, "training_step", return_value={"status": "completed"}) as step, mock.patch.object(build_pgo, "lsp_session", return_value=(0, "session")):
                build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, False, Path(tmp))
            names = [call.args[0] for call in step.call_args_list]
        self.assertIn("check jaifmt", names)
        self.assertFalse(any("native" in name or "build jaifmt" in name for name in names))


class LspSessionTests(unittest.TestCase):
    def test_session_retains_stderr_and_partial_timeout_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "jailsp.log"
            result = mock.Mock(returncode=1, stdout=b"response", stderr=b"failure")
            with mock.patch.object(build_pgo.subprocess, "run", return_value=result):
                code, _ = build_pgo.lsp_session("jailsp", {}, output)
            self.assertEqual(code, 1)
            self.assertEqual(output.read_bytes(), b"responsefailure")
            timeout = subprocess.TimeoutExpired(["jailsp"], 900, output=b"partial", stderr=b"stuck")
            with mock.patch.object(build_pgo.subprocess, "run", side_effect=timeout):
                code, _ = build_pgo.lsp_session("jailsp", {}, output)
            self.assertEqual(code, "timeout")
            self.assertEqual(output.read_bytes(), b"partialstuck")

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
