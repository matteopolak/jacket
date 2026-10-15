#!/usr/bin/env python3
"""Exercise real native SMTP/MIME against this checkout's loopback receiver only."""
import argparse
import base64
import email.parser
import email.policy
import email.utils
import os
from pathlib import Path
import socket
import subprocess
import threading

from portable_toolchain import default_compiler, checked_compiler

ROOT = Path(__file__).resolve().parents[1]


def receive(listener, mode, outcome):
    try:
        connection, _ = listener.accept()
        with connection:
            connection.settimeout(10)
            stream = connection.makefile("rwb", buffering=0)
            connection.sendall(b"220 portable-test ESMTP\r\n")
            recipients = []
            sender = None
            authenticated = False
            auth_pending = False
            payload = None
            while True:
                command = stream.readline(65536)
                if not command:
                    break
                upper = command.upper()
                if auth_pending or upper.startswith(b"AUTH PLAIN"):
                    encoded = command.strip() if auth_pending else command.strip()[10:].strip()
                    if not encoded:
                        auth_pending = True
                        connection.sendall(b"334 \r\n")
                        continue
                    assert mode == "auth" and base64.b64decode(encoded, validate=True) == b"\x00portable-user\x00portable-password"
                    authenticated = True
                    auth_pending = False
                    connection.sendall(b"235 authentication accepted\r\n")
                elif upper.startswith((b"EHLO ", b"HELO ")):
                    connection.sendall(b"250-portable-test\r\n250-AUTH PLAIN\r\n250 8BITMIME\r\n" if mode == "auth" else b"250-portable-test\r\n250 8BITMIME\r\n")
                elif upper.startswith(b"MAIL FROM:"):
                    sender = command.strip().split(b":", 1)[1].split()[0]
                    connection.sendall(b"250 sender accepted\r\n")
                elif upper.startswith(b"RCPT TO:"):
                    recipients.append(command.strip().split(b":", 1)[1])
                    rejected = mode == "reject" and b"<bcc@example.invalid>" in command
                    connection.sendall(b"550 recipient rejected\r\n" if rejected else b"250 recipient accepted\r\n")
                elif upper == b"DATA\r\n":
                    connection.sendall(b"354 send data\r\n")
                    lines = []
                    while True:
                        line = stream.readline(65536)
                        if line == b".\r\n":
                            break
                        if not line:
                            raise RuntimeError("SMTP data ended without its terminator")
                        lines.append(line[1:] if line.startswith(b"..") else line)
                    payload = b"".join(lines)
                    connection.sendall(b"250 message accepted\r\n")
                elif upper == b"QUIT\r\n":
                    connection.sendall(b"221 closing\r\n")
                    break
                else:
                    raise RuntimeError(f"unexpected SMTP command: {command!r}")
            outcome.append((sender, recipients, payload, authenticated))
    except BaseException as error:
        outcome.append(error)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=default_compiler())
    parser.add_argument("--stdlib", type=Path, default=ROOT / "target/portable-integration-stdlib")
    parser.add_argument("--native", action="store_true")
    args = parser.parse_args()
    compiler = checked_compiler(args.jaic, parser)
    output = ROOT / "target/mail-tests"
    output.mkdir(parents=True, exist_ok=True)
    attachment = output / "attachment.bin"
    content = b"\x00portable\xffattachment\r\n"
    attachment.write_bytes(content)
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve(strict=True)))
    source = ROOT / "tests/portable-stdlib/integration/mail.jai"
    executable = output / "mail-test"
    if args.native:
        subprocess.run([str(compiler), str(source), "-exe", str(executable)], env=env, cwd=ROOT, check=True)
    for native in ([False, True] if args.native else [False]):
        for mode in ["plain", "html", "auth", "reject", "tls"]:
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                listener.listen(1)
                listener.settimeout(10)
                port = listener.getsockname()[1]
                outcome = []
                server = threading.Thread(target=receive, args=(listener, mode, outcome), daemon=True)
                server.start()
                command = [str(executable)] if native else [str(compiler), "-interp", str(source), "--"]
                result = subprocess.run(command + [str(port), str(attachment), mode], cwd=ROOT,
                                        env=env, capture_output=True, text=True, timeout=20)
                server.join(timeout=11)
                if server.is_alive():
                    raise RuntimeError("loopback SMTP receiver did not finish")
                if result.returncode or result.stdout != "ok\n" or result.stderr:
                    raise RuntimeError(f"mail client failed: {result.returncode}: {result.stdout}{result.stderr}")
                if len(outcome) != 1 or isinstance(outcome[0], BaseException):
                    raise RuntimeError(f"SMTP receiver failed: {outcome}")
                sender, recipients, payload, authenticated = outcome[0]
                if mode in {"reject", "tls"}:
                    assert payload is None, "failed transfer delivered a message"
                else:
                    assert sender == b"<sender@example.invalid>"
                    assert authenticated == (mode == "auth")
                    assert set(recipients) == {b"<to@example.invalid>", b"<second@example.invalid>", b"<cc@example.invalid>", b"<bcc@example.invalid>"}
                    assert payload is not None
                    message = email.parser.BytesParser(policy=email.policy.default).parsebytes(payload)
                    assert str(message["Subject"]) == "Portable stdlib"
                    assert message["Bcc"] is None and message["Cc"] is not None and message["Reply-To"] is not None
                    assert email.utils.parseaddr(str(message["From"]))[1] == "sender@example.invalid"
                    assert email.utils.parseaddr(str(message["Reply-To"]))[1] == "reply@example.invalid"
                    assert {address for _, address in email.utils.getaddresses(message.get_all("To"))} == {"to@example.invalid", "second@example.invalid"}
                    body = message.get_body(preferencelist=("html", "plain"))
                    assert body.get_content_type() == ("text/plain" if mode == "plain" else "text/html")
                    assert body.get_payload(decode=True) == "portable π body\r\n".encode()
                    attachments = list(message.iter_attachments())
                    assert len(attachments) == 1 and attachments[0].get_filename() == attachment.name
                    assert attachments[0].get_payload(decode=True) == content
            print(f"passed loopback SMTP mode={mode}, native={native}", flush=True)


if __name__ == "__main__":
    main()
