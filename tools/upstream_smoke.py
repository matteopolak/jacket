#!/usr/bin/env python3
"""Build pinned open-source Jai projects with their own entry points and run them headlessly.

Checks live in tools/upstream-smoke.json (docs/tools/third-party-smoke-test.md). Each runs in a
scratch copy of its project under corpus/upstream, so a build that writes into its tree never
changes the pinned sources. Exits 1 when a check that was not skipped fails.
"""
import argparse
import json
import os
import platform
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM = ROOT / "corpus/upstream"
CHECKS = ROOT / "tools/upstream-smoke.json"
WINDOWS = sys.platform == "win32"


def host_platform():
    machine = platform.machine().lower()
    arm = machine in ("arm64", "aarch64")
    if sys.platform == "darwin":
        return "macos-arm64" if arm else "macos-x64"
    if WINDOWS:
        return "windows-arm64" if arm else "windows-x64"
    return "linux-arm64" if arm else "linux-x64"


class Failure(Exception):
    pass


def tail(text, lines=25):
    return "\n".join(text.strip().splitlines()[-lines:])


def expand(argv, ctx):
    """Substitute {jaic} and {python}; a program name that is only found with `.exe` on Windows."""
    out = []
    for a in argv:
        out.append(a.replace("{jaic}", ctx["jaic"]).replace("{python}", sys.executable)
                    .replace("{work}", str(ctx["work"])))
    prog = out[0]
    if WINDOWS and not prog.lower().endswith(".exe") and not Path(ctx["cwd"], prog).exists() \
            and Path(ctx["cwd"], prog + ".exe").exists():
        out[0] = prog + ".exe"
    if not os.path.isabs(out[0]) and ("/" in out[0] or "\\" in out[0] or Path(ctx["cwd"], out[0]).exists()):
        out[0] = str(Path(ctx["cwd"], out[0]).resolve())
    return out


def check_output(text, step, label):
    for needle in step.get("contains", []):
        if needle not in text:
            raise Failure(f"{label}: output lacks {needle!r}\n{tail(text)}")
    last = 0
    for needle in step.get("ordered", []):
        at = text.find(needle, last)
        if at < 0:
            raise Failure(f"{label}: output lacks {needle!r} after offset {last}\n{tail(text)}")
        last = at + len(needle)
    for needle in step.get("not_contains", []):
        if needle in text:
            raise Failure(f"{label}: output has {needle!r}\n{tail(text)}")


def run_step(step, ctx):
    argv = expand(step["argv"], ctx)
    env = {**ctx["env"], **step.get("env", {})}
    try:
        done = subprocess.run(argv, cwd=ctx["cwd"], env=env, input=step.get("stdin", ""),
                              capture_output=True, text=True, errors="replace",
                              timeout=step.get("timeout", 300))
    except subprocess.TimeoutExpired as e:
        raise Failure(f"{' '.join(argv[:3])}: timed out after {step.get('timeout', 300)}s\n"
                      f"{tail((e.stdout or b'').decode(errors='replace') if isinstance(e.stdout, bytes) else (e.stdout or ''))}")
    text = done.stdout + done.stderr
    want = step.get("exit", 0)
    if done.returncode != want:
        raise Failure(f"{' '.join(argv[:3])}: exit {done.returncode}, expected {want}\n{tail(text)}")
    check_output(text.replace("\r\n", "\n"), step, " ".join(argv[:3]))
    for name in step.get("outputs", []):
        if not any(Path(ctx["cwd"], n).exists() for n in (name, name + ".exe")):
            raise Failure(f"{name}: not produced")
    return text


# --- Screenshots ------------------------------------------------------------------------------
# Comparing the screen before and after the program starts proves it drew something, whatever the
# desktop or wallpaper looks like.

def read_bmp(path):
    data = Path(path).read_bytes()
    offset = struct.unpack_from("<I", data, 10)[0]
    width, height, _, bpp = struct.unpack_from("<iiHH", data, 18)
    top_down = height < 0
    height = abs(height)
    step = bpp // 8
    stride = (width * step + 3) & ~3
    return width, height, bpp, data, offset, stride, top_down


def sample_bmp(path, cols=96, rows=54):
    width, height, bpp, data, offset, stride, top_down = read_bmp(path)
    if bpp not in (24, 32):
        raise Failure(f"unsupported bitmap depth {bpp}")
    step = bpp // 8
    out = []
    for r in range(rows):
        y = r * height // rows
        row = y if top_down else height - 1 - y
        base = offset + row * stride
        for c in range(cols):
            x = c * width // cols
            out.append(bytes(data[base + x * step: base + x * step + 3]))
    return out


def sample_xwd(path, cols=96, rows=54):
    data = Path(path).read_bytes()
    h = struct.unpack_from(">25I", data, 0)
    header, width, height, bpp, stride, ncolors = h[0], h[4], h[5], h[11], h[12], h[19]
    base = header + ncolors * 12
    step = bpp // 8
    out = []
    for r in range(rows):
        y = r * height // rows
        for c in range(cols):
            x = c * width // cols
            at = base + y * stride + x * step
            out.append(bytes(data[at: at + step]))
    return out


def capture(dest):
    """Save the whole screen; returns a function that reads it as sampled pixels, or None."""
    try:
        if sys.platform == "darwin":
            path = dest.with_suffix(".bmp")
            subprocess.run(["screencapture", "-x", "-t", "bmp", str(path)], check=True, timeout=60)
            return lambda: sample_bmp(path)
        if WINDOWS:
            path = dest.with_suffix(".bmp")
            script = ("Add-Type -AssemblyName System.Windows.Forms,System.Drawing;"
                      "$b=[Windows.Forms.SystemInformation]::VirtualScreen;"
                      "$bmp=New-Object Drawing.Bitmap $b.Width,$b.Height;"
                      "$g=[Drawing.Graphics]::FromImage($bmp);"
                      "$g.CopyFromScreen($b.Location,[Drawing.Point]::Empty,$b.Size);"
                      f"$bmp.Save('{path}',[Drawing.Imaging.ImageFormat]::Bmp)")
            subprocess.run(["powershell", "-NoProfile", "-Command", script], check=True, timeout=120)
            return lambda: sample_bmp(path)
        path = dest.with_suffix(".xwd")
        with open(path, "wb") as f:
            subprocess.run(["xwd", "-root", "-silent"], check=True, stdout=f, timeout=60)
        return lambda: sample_xwd(path)
    except (OSError, subprocess.SubprocessError, Failure) as e:
        print(f"    screenshot unavailable: {e}")
        return None


def stop(proc):
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()


def read_pipe_text(path):
    try:
        return Path(path).read_text(errors="replace")
    except OSError:
        return ""


def gui_step(step, ctx):
    """Start a windowed program, let it run, require it to stay up and draw, then stop it."""
    argv = expand(step["argv"], ctx)
    env = {**ctx["env"], **step.get("env", {})}
    if not WINDOWS:
        # Keep the program's settings out of the real home directory.
        home = ctx["work"] / "home"
        home.mkdir(exist_ok=True)
        env.update(HOME=str(home), CFFIXED_USER_HOME=str(home), XDG_CONFIG_HOME=str(home / ".config"),
                   XDG_DATA_HOME=str(home / ".local/share"), XDG_CACHE_HOME=str(home / ".cache"))
    log = ctx["work"] / "gui.log"
    shots = ctx["shots"]
    # Software OpenGL for Windows runners: Windows looks for opengl32.dll beside the program first.
    for dll in ctx["dlls"]:
        target = Path(argv[0]).parent / dll.name
        if not target.exists():
            shutil.copyfile(dll, target)
    before = capture(shots / f"{ctx['id']}-before") if step.get("screenshot", True) else None
    base = before() if before else None
    with open(log, "wb") as out:
        proc = subprocess.Popen(argv, cwd=ctx["cwd"], env=env, stdin=subprocess.DEVNULL, stdout=out,
                                stderr=subprocess.STDOUT)
    try:
        seconds = step.get("seconds", 8)
        deadline = time.time() + seconds
        while time.time() < deadline:
            if proc.poll() is not None:
                break
            time.sleep(0.25)
        if proc.poll() is not None and not (step.get("may_exit") and proc.returncode == 0):
            raise Failure(f"{argv[0]}: exited with {proc.returncode} after {seconds - (deadline - time.time()):.1f}s\n"
                          f"{tail(read_pipe_text(log))}")
        if before:
            after = capture(shots / f"{ctx['id']}-after")
            if after:
                now = after()
                changed = sum(1 for a, b in zip(base, now) if a != b) / len(now)
                colors = len(set(now))
                print(f"    screen: {changed:.1%} of samples changed, {colors} colors")
                if changed < step.get("min_changed", 0.005) or colors < 4:
                    raise Failure(f"{argv[0]}: the screen did not change ({changed:.2%} changed, {colors} colors)")
    finally:
        stop(proc)
    text = read_pipe_text(log).replace("\r\n", "\n")
    check_output(text, step, argv[0])
    return text


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def serve_step(step, ctx):
    """Start a server, fetch URLs from it, and check the bodies."""
    port = step.get("port") or free_port()
    argv = [a.replace("{port}", str(port)) for a in expand(step["argv"], ctx)]
    log = ctx["work"] / "serve.log"
    with open(log, "wb") as out:
        proc = subprocess.Popen(argv, cwd=ctx["cwd"], env={**ctx["env"], **step.get("env", {})},
                                stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT)
    try:
        deadline = time.time() + step.get("timeout", 30)
        body = None
        while time.time() < deadline:
            if proc.poll() is not None:
                raise Failure(f"{argv[0]}: exited with {proc.returncode}\n{tail(read_pipe_text(log))}")
            try:
                url = f"http://127.0.0.1:{port}{step.get('get', '/')}"
                with urllib.request.urlopen(url, timeout=5) as r:
                    body = r.read().decode(errors="replace")
                    status = r.status
                break
            except OSError:
                time.sleep(0.5)
        if body is None:
            raise Failure(f"{argv[0]}: no answer on port {port}\n{tail(read_pipe_text(log))}")
        if status != step.get("status", 200):
            raise Failure(f"{argv[0]}: HTTP {status}")
        check_output(body, step, "response")
        return body
    finally:
        stop(proc)


def lsp_step(step, ctx):
    """Send `initialize` over stdio and read the server's reply."""
    argv = expand(step["argv"], ctx)
    request = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                          "params": {"processId": None, "rootUri": None, "capabilities": {}}})
    shutdown = json.dumps({"jsonrpc": "2.0", "id": 2, "method": "shutdown"})
    frame = lambda s: f"Content-Length: {len(s.encode())}\r\n\r\n{s}".encode()
    proc = subprocess.Popen(argv, cwd=ctx["cwd"], env=ctx["env"], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        proc.stdin.write(frame(request) + frame(shutdown))
        proc.stdin.flush()
        replies = []
        for _ in range(20):
            header = b""
            while not header.endswith(b"\r\n\r\n"):
                c = proc.stdout.read(1)
                if not c:
                    raise Failure(f"{argv[0]}: closed without answering\n{tail(proc.stderr.read().decode(errors='replace'))}\n"
                                  f"{tail(chr(10).join(replies))}")
                header += c
            length = int(header.split(b"Content-Length:")[1].split(b"\r\n")[0])
            text = proc.stdout.read(length).decode(errors="replace")
            replies.append(text)
            if '"id": 1' in text or '"id":1' in text:  # the reply to `initialize`, after any log messages
                check_output(text, step, "initialize response")
                return text
        raise Failure(f"{argv[0]}: no reply to initialize\n{tail(chr(10).join(replies))}")
    finally:
        stop(proc)


STEPS = {"run": run_step, "gui": gui_step, "serve": serve_step, "lsp": lsp_step}


def skip_reason(check, plat):
    if plat in check.get("skip", {}):
        return check["skip"][plat]
    if "only" in check and plat not in check["only"]:
        return check.get("only_reason", "not supported on this platform")
    return None


def run_check(check, args, plat, shots):
    work = Path(tempfile.mkdtemp(prefix=f"smoke-{check['id']}-"))
    try:
        for name in [check["project"], *check.get("copy", [])]:
            shutil.copytree(UPSTREAM / name, work / name, symlinks=True)
        for name in check.get("link", []):
            (work / name).symlink_to(UPSTREAM / name, target_is_directory=True)
        for dest, source in check.get("files", {}).items():
            shutil.copyfile(ROOT / source, work / dest)
        cwd = work / check["project"] / check.get("dir", "")
        env = dict(os.environ)
        env.update(check.get("env", {}))
        ctx = {"jaic": str(Path(args.jaic).resolve()), "work": work, "cwd": cwd, "env": env,
               "shots": shots, "id": check["id"], "dlls": args.dlls}
        for step in check["steps"]:
            kind = step.get("kind", "run")
            if "only" in step and plat not in step["only"]:
                continue
            if kind == "jaic":
                step = {**step, "argv": ["{jaic}", *step["argv"]], "outputs": step.get("outputs", [])}
                kind = "run"
            label = step.get("name") or " ".join(step["argv"][:4])
            started = time.time()
            text = STEPS[kind](step, {**ctx, "cwd": ctx["cwd"] / step.get("cwd", "")})
            print(f"    ok {label} ({time.time() - started:.0f}s)", flush=True)
            if args.verbose:
                print(tail(text or "", 40))
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--jaic", required=True)
    ap.add_argument("--platform", default=host_platform())
    ap.add_argument("--filter", action="append", default=[], help="only checks whose id contains this")
    ap.add_argument("--screenshots", default=str(ROOT / "target/smoke-screenshots"))
    ap.add_argument("--checks", default=str(CHECKS))
    ap.add_argument("--dlls", default=None, help="a directory of DLLs (Mesa's opengl32.dll) copied beside windowed programs")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--verbose", "-v", action="store_true", help="print each step's output")
    ap.add_argument("--projects", action="store_true", help="print the corpus repositories the checks use")
    args = ap.parse_args()
    args.dlls = sorted(Path(args.dlls).glob("*.dll")) if args.dlls else []
    checks = json.loads(Path(args.checks).read_text())["checks"]
    if args.filter:
        checks = [c for c in checks if any(f in c["id"] for f in args.filter)]
    if args.projects:
        names = sorted({n for c in checks for n in [c["project"], *c.get("copy", []), *c.get("link", [])]})
        print(" ".join(n.replace("--", "/", 1) for n in names))
        return 0
    if args.list:
        for c in checks:
            print(c["id"], skip_reason(c, args.platform) or "")
        return 0
    if not os.environ.get("JAIC_NATIVE_LIBS") and sys.platform in ("darwin", "linux", "win32"):
        sys.path.insert(0, str(ROOT / "tools"))
        import build_native_libs
        plat = args.platform if WINDOWS else None
        if build_native_libs.missing(plat):
            subprocess.run([sys.executable, str(ROOT / "tools/build_native_libs.py"),
                            *(["--platform", plat] if plat else [])], check=True)
        os.environ["JAIC_NATIVE_LIBS"] = str(build_native_libs.output_dir(plat))
    shots = Path(args.screenshots)
    shots.mkdir(parents=True, exist_ok=True)
    results = []
    for check in checks:
        reason = skip_reason(check, args.platform)
        print(f"== {check['id']}", flush=True)
        if reason:
            print(f"    SKIP {reason}")
            results.append((check["id"], "skip", reason))
            continue
        try:
            run_check(check, args, args.platform, shots)
            results.append((check["id"], "pass", ""))
        except Failure as e:
            print(f"    FAIL {e}")
            results.append((check["id"], "FAIL", str(e).splitlines()[0]))
        except Exception as e:  # noqa: BLE001 - a broken check must not hide the others
            print(f"    ERROR {type(e).__name__}: {e}")
            results.append((check["id"], "FAIL", f"{type(e).__name__}: {e}"))
    print(f"\n{args.platform}:")
    for id_, status, note in results:
        print(f"  {status:5} {id_}" + (f"  {note}" if note else ""))
    return 1 if any(s == "FAIL" for _, s, _ in results) else 0


if __name__ == "__main__":
    sys.exit(main())
