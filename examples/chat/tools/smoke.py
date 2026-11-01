"""Exercise the real Jai programs; Python only orchestrates processes and checks transcripts."""
import os
import ctypes
import errno
from pathlib import Path
import shutil
import signal
import select
import pty
import socket
import subprocess
import sys
import time

work = Path(sys.argv[1])
chat = str(work / "chat")
processes = []
logs = []


def port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def start(name, *args):
    log = open(work / (name + ".out"), "w+")
    logs.append(log)
    proc = subprocess.Popen([chat, *args], stdout=log, stderr=subprocess.STDOUT)
    processes.append(proc)
    return proc


def wait_text(name, needle, proc):
    until = time.monotonic() + 10
    while time.monotonic() < until:
        text = (work / (name + ".out")).read_text()
        if needle in text:
            return text
        if proc.poll() is not None:
            raise AssertionError(f"{name} exited {proc.returncode}: {text}")
        time.sleep(0.03)
    raise AssertionError(f"{name} did not print {needle}: {text}")


def run(*args, good=True):
    result = subprocess.run([chat, *args], capture_output=True, text=True, timeout=20)
    if good:
        assert result.returncode == 0, (args, result.stdout, result.stderr)
    else:
        assert result.returncode != 0, args
    return result.stdout


def script(name, text, address, room="lobby", roots=None):
    path = work / (name + ".script")
    path.write_text(text)
    args = ["client", "--address", address, "--nick", name, "--room", room, "--script", str(path)]
    if roots:
        args += ["--roots", str(roots)]
    return run(*args)


def stop(proc, sig=signal.SIGTERM):
    proc.send_signal(sig)
    assert proc.wait(timeout=8) == 0


try:
    smtp_port = port()
    sink = start("sink", "smtp-sink", "--port", str(smtp_port), "--file", str(work / "digest.eml"))
    wait_text("sink", "SMTP READY", sink)
    address = f"127.0.0.1:{port()}"
    channel = f"chat-smoke-{os.getpid()}"
    server_args = ["serve", "--listen", address, "--file", str(work / "history.log"),
                   "--history", "3", "--rate", "100000", "--channel", channel,
                   "--smtp-host", "127.0.0.1", "--smtp-port", str(smtp_port)]
    server = start("server", *server_args)
    wait_text("server", "LISTEN", server)
    script("offline", "EMAIL offline@example.test\nQUIT\n", address)
    bot = start("bot", "bot", "--address", address, "--duration", "5000")
    wait_text("bot", "BOT READY", bot)
    watch = work / "watch.script"
    watch.write_text("WAIT 3000\nQUIT\n")
    watcher = start("watcher", "client", "--address", address, "--nick", "watcher", "--script", str(watch))
    time.sleep(0.15)
    output = script("alice", "SAY hello @offline\nSAY !ping\nWAIT 400\nDM watcher private-note\nJOIN other\nSAY isolated\nLEAVE\nEXPECT-ERROR SAY nowhere\nJOIN lobby\nEXPECT-ERROR NICK bob\nEXPECT-ERROR DM absent missing\nHISTORY\nQUIT\n", address)
    assert "MSG lobby bot pong" in output, output
    assert "HISTORY lobby alice hello @offline" in output, output
    assert "ERR join-first" in output, output
    # Sandboxed macOS sessions can forbid POSIX shared-memory creation. Verify the
    # exact OS error before skipping this facility; other failures still fail smoke.
    lib = ctypes.CDLL(None, use_errno=True)
    probe_name = ("/chat-probe-" + str(os.getpid())).encode()
    probe_fd = lib.shm_open(probe_name, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
    probe_errno = ctypes.get_errno()
    if probe_fd < 0 and probe_errno in (errno.EPERM, errno.EACCES, errno.ENOSYS):
        print("chat smoke: stats skipped (shm_open: " + os.strerror(probe_errno) + ")")
    else:
        assert probe_fd >= 0, ("shm_open", probe_errno)
        os.close(probe_fd)
        lib.shm_unlink(probe_name)
        stats = run("stats", "--channel", channel)
        assert "connections=" in stats and "total=4" in stats, stats
    assert watcher.wait(timeout=8) == 0
    transcript = (work / "watcher.out").read_text()
    assert "PRIVATE alice private-note" in transcript, transcript
    assert "JOINED lobby alice" in transcript and "LEFT lobby alice" in transcript, transcript
    assert "isolated" not in transcript, transcript
    assert sink.wait(timeout=8) == 0
    digest = (work / "digest.eml").read_text()
    assert "hello @offline" in digest and "offline@example.test" in digest, digest
    load = run("load", "--address", address, "--clients", "4", "--rounds", "6")
    assert "messages=24" in load and "p50=" in load and "p99=" in load and "failures=0" in load, load
    # Exercise the actual interactive input and network fibers through a pseudo-terminal.
    master, slave = pty.openpty()
    terminal = subprocess.Popen([chat, "client", "--address", address, "--nick", "terminal",
                                 "--room", "terminal"], stdin=slave, stdout=slave, stderr=slave)
    processes.append(terminal)
    os.close(slave)
    terminal_text = b""

    def terminal_until(needle):
        global terminal_text
        end = time.monotonic() + 5
        while needle not in terminal_text and time.monotonic() < end:
            readable, _, _ = select.select([master], [], [], 0.1)
            if readable:
                terminal_text += os.read(master, 8192)
        assert needle in terminal_text, terminal_text

    try:
        terminal_until(b"> ")
        os.write(master, b"terminal-message\n")
        terminal_until(b"MSG terminal terminal terminal-message")
        assert b"\x1b[1;22r" in terminal_text and b"\x1b7" in terminal_text, terminal_text
        os.write(master, b"/QUIT\n")
        assert terminal.wait(timeout=5) == 0
    finally:
        os.close(master)
    # Keep a reader parked during shutdown; half a final command must never execute.
    idle = socket.create_connection(tuple(["127.0.0.1", int(address.rsplit(":", 1)[1])]))
    idle.sendall(b"NICK interrupted\nJOIN lobby\nSAY unfinished")
    time.sleep(0.05)
    stop(server)
    idle.close()
    assert "STOPPED" in (work / "server.out").read_text()
    bot.wait(timeout=8)
    with (work / "history.log").open("a") as log:
        log.write("torn-tail")
    restart = start("restart", *server_args)
    wait_text("restart", "LISTEN", restart)
    history = script("reader", "HISTORY\nQUIT\n", address)
    assert "HISTORY lobby bot pong" in history and "HISTORY lobby alice !ping" in history, history
    stop(restart, signal.SIGINT)
    assert "torn-tail" not in (work / "history.log").read_text()
    assert "unfinished" not in (work / "history.log").read_text()
    # A corrupt complete line must prevent the server from starting.
    (work / "bad.log").write_text("lobby\talice\tbad\tchecksum\n")
    run("serve", "--file", str(work / "bad.log"), good=False)
    # The limiter is observable over the real protocol, with its default burst of twenty.
    rate_address = f"127.0.0.1:{port()}"
    limited = start("limited", "serve", "--listen", rate_address, "--rate", "1", "--file", str(work / "limited.log"))
    wait_text("limited", "LISTEN", limited)
    rate = script("rate", "PING\n" * 18 + "EXPECT-ERROR PING\nWAIT 1100\nQUIT\n", rate_address)
    assert "ERR rate-limit" in rate, rate
    stop(limited)
    if shutil.which("openssl"):
        cert, key = work / "cert.pem", work / "key.pem"
        subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
                        "-subj", "/CN=localhost", "-addext", "subjectAltName=DNS:localhost", "-keyout", str(key),
                        "-out", str(cert)], check=True, capture_output=True)
        secure_address = f"127.0.0.1:{port()}"
        secure = start("tls", "serve", "--listen", secure_address, "--file", str(work / "tls.log"),
                       "--tls-cert", str(cert), "--tls-key", str(key))
        wait_text("tls", "LISTEN", secure)
        tls = script("secure", "SAY encrypted\nHISTORY\nQUIT\n", secure_address, roots=cert)
        assert "HISTORY lobby secure encrypted" in tls, tls
        # A different CA must fail verification.
        wrong = work / "wrong.pem"
        subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
                        "-subj", "/CN=other", "-keyout", str(work / "wrong.key"), "-out", str(wrong)], check=True, capture_output=True)
        run("client", "--address", secure_address, "--nick", "badca", "--roots", str(wrong),
            "--script", str(work / "secure.script"), good=False)
        stop(secure)
    else:
        print("chat smoke: TLS skipped (openssl is absent)")
    print("chat smoke: ok (rooms, private messages, bot, recovery, limiter, load, SMTP, TLS; see skips above)")
    print(load.strip())
except Exception:
    for path in work.glob("*.out"):
        print(f"--- {path.name} ---\n{path.read_text()}", file=sys.stderr)
    raise
finally:
    for proc in processes:
        if proc.poll() is None:
            proc.kill()
        proc.wait()
    for log in logs:
        log.close()
