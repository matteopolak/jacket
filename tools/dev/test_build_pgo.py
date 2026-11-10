import json
import tempfile
import time
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
    def test_timeout_kills_training_children_and_grandchildren(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            ready, survived = root / "ready", root / "survived"
            grandchild = (f"from pathlib import Path; import time; Path({str(ready)!r}).touch(); "
                          f"time.sleep(1); Path({str(survived)!r}).touch()")
            child = f"import subprocess, sys, time; subprocess.Popen([sys.executable, '-c', {grandchild!r}]); time.sleep(2)"
            parent = f"import subprocess, sys, time; subprocess.Popen([sys.executable, '-c', {child!r}]); time.sleep(2)"
            result = build_pgo.training_step("tree timeout", [build_pgo.sys.executable, "-c", parent],
                                             None, root / "step.log", 0.5)
            self.assertEqual(result["exit_code"], "timeout")
            self.assertTrue(ready.exists(), "grandchild must start before timeout")
            time.sleep(0.8)
            self.assertFalse(survived.exists(), "grandchild outlived training timeout")
            # Synthetic descendants exit on their own within two seconds even on a regression.

    def test_profile_input_is_not_deleted_by_training_evidence_reset(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp)
            profile = target / "pgo" / "training" / "saved.profdata"
            profile.parent.mkdir(parents=True)
            profile.write_bytes(b"user supplied profile")
            argv = ["build_pgo.py", "--target-dir", str(target), "--target", "x86_64-unknown-linux-gnu",
                    "--llvm", "none", "--profile", str(profile)]
            with mock.patch.object(build_pgo.sys, "argv", argv), mock.patch.object(build_pgo, "cargo_build", side_effect=RuntimeError("build entered")) as build:
                with self.assertRaisesRegex(SystemExit, "training evidence"):
                    build_pgo.main()
            build.assert_not_called()
            self.assertEqual(profile.read_bytes(), b"user supplied profile")

    def test_invalid_profile_is_rejected_before_existing_evidence_reset(self):
        for kind in ("missing", "directory"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                target = Path(tmp)
                evidence = target / "pgo" / "training" / "old.log"
                evidence.parent.mkdir(parents=True)
                evidence.write_bytes(b"previous evidence")
                supplied = target / "missing.profdata" if kind == "missing" else target
                argv = ["build_pgo.py", "--target-dir", str(target), "--target", "x86_64-unknown-linux-gnu",
                        "--llvm", "none", "--profile", str(supplied)]
                with mock.patch.object(build_pgo.sys, "argv", argv), mock.patch.object(build_pgo, "cargo_build", side_effect=RuntimeError("build entered")) as build:
                    with self.assertRaisesRegex(SystemExit, "--profile"):
                        build_pgo.main()
                build.assert_not_called()
                self.assertEqual(evidence.read_bytes(), b"previous evidence")

    def test_unreadable_profile_leaves_existing_evidence_intact(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            training = root / "training"
            training.mkdir()
            evidence = training / "old.log"
            evidence.write_bytes(b"previous evidence")
            profile = root / "saved.profdata"
            profile.write_bytes(b"profile")
            with mock.patch.object(Path, "open", side_effect=PermissionError("read denied")):
                with self.assertRaisesRegex(SystemExit, "read denied"):
                    build_pgo.profile_input(profile, training)
            self.assertEqual(evidence.read_bytes(), b"previous evidence")
            self.assertEqual(profile.read_bytes(), b"profile")

    def test_profile_aliases_and_invalid_inputs_preserve_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            training = root / "training"
            training.mkdir()
            evidence = training / "old.log"
            evidence.write_bytes(b"previous evidence")
            outside = root / "saved.profdata"
            outside.write_bytes(b"profile")
            inside = training / "inside.profdata"
            inside.write_bytes(b"inside")
            outside_alias = root / "inside-alias.profdata"
            outside_alias.symlink_to(inside)
            for path in (inside, outside_alias):
                with self.subTest(path=path):
                    with self.assertRaisesRegex(SystemExit, "training evidence"):
                        build_pgo.profile_input(path, training)
            for path in (root / "missing.profdata", root):
                with self.subTest(path=path):
                    with self.assertRaisesRegex(SystemExit, "--profile"):
                        build_pgo.profile_input(path, training)
            self.assertEqual(build_pgo.profile_input(outside, training), outside.resolve())
            self.assertEqual(evidence.read_bytes(), b"previous evidence")
            self.assertEqual(outside.read_bytes(), b"profile")
            self.assertEqual(inside.read_bytes(), b"inside")
            self.assertTrue(outside_alias.is_symlink())

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
                kwargs["output"].write(b"x" * 20000 + b"\nmissing backend library\n")
                return mock.Mock(returncode=1, timed_out=False)
            with mock.patch.object(build_pgo, "run_measured", side_effect=fail), mock.patch.object(build_pgo, "log") as log:
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
            with mock.patch.object(build_pgo, "run_measured", return_value=mock.Mock(returncode=1, timed_out=False)):
                result = build_pgo.training_step("lint", ["jailint"], {}, Path(tmp) / "lint.log", 30, expected=(0, 1))
        self.assertEqual(result["status"], "completed")
        self.assertEqual(result["exit_code"], 1)

    def test_training_failures_stop_the_build(self):
        build_pgo.fail_on([])
        with self.assertRaises(SystemExit) as stop:
            build_pgo.fail_on(["sweep (check and run) (exit 1)"])
        self.assertIn("sweep (check and run) (exit 1)", str(stop.exception))

    def test_lint_steps_expect_a_clean_exit_and_only_the_stdlib_subset(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = mock.Mock(jobs=2, step_timeout=30, verbose=False)
            step = lambda name, cmd, env, path, timeout, **kwargs: {"name": name, "command": list(map(str, cmd)), "status": "completed"}
            with mock.patch.object(build_pgo, "training_step", side_effect=step) as run, mock.patch.object(build_pgo, "lsp_session", return_value=(0, "session")):
                build_pgo.train({"jaic": "jaic", "jailint": "jailint", "jailsp": "jailsp"}, args, {}, False, Path(tmp))
            report = json.loads((Path(tmp) / "summary.json").read_text())
        for call in run.call_args_list:
            self.assertNotIn("expected", call.kwargs)
        lint = [r for r in report["steps"] if r["name"].startswith("jailint")]
        self.assertEqual(len(lint), 2)
        self.assertNotIn("stdlib", lint[0]["command"])
        for path in build_pgo.LINT_TRAINING_PATHS:
            self.assertTrue((build_pgo.ROOT / path).is_dir(), path)
        self.assertIn("--allow-incomplete", lint[1]["command"])

    def test_timeout_keeps_partial_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "timeout.log"
            def timeout(cmd, **kwargs):
                kwargs["output"].write(b"partial output\n")
                return mock.Mock(returncode=-9, timed_out=True)
            with mock.patch.object(build_pgo, "run_measured", side_effect=timeout):
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
            def fail(cmd, **kwargs):
                kwargs["output"].write(b"responsefailure")
                return mock.Mock(returncode=1, timed_out=False)
            with mock.patch.object(build_pgo, "run_measured", side_effect=fail):
                code, _ = build_pgo.lsp_session("jailsp", {}, output)
            self.assertEqual(code, 1)
            self.assertEqual(output.read_bytes(), b"responsefailure")
            def timeout(cmd, **kwargs):
                kwargs["output"].write(b"partialstuck")
                return mock.Mock(returncode=-9, timed_out=True)
            with mock.patch.object(build_pgo, "run_measured", side_effect=timeout):
                code, _ = build_pgo.lsp_session("jailsp", {}, output)
            self.assertEqual(code, "timeout")
            self.assertEqual(output.read_bytes(), b"partialstuck")

    def test_session_is_framed_and_ends_with_exit(self):
        done = mock.Mock(returncode=0, timed_out=False)
        with mock.patch.object(build_pgo, "run_measured", return_value=done) as run:
            code, _ = build_pgo.lsp_session("jailsp", {})
        self.assertEqual(code, 0)
        data, messages = run.call_args.kwargs["stdin_bytes"], []
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
