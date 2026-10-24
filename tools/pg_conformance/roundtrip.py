#!/usr/bin/env python3
"""Runs the binary round trip (stdlib/Extensions/Postgres/tests/conformance/roundtrip.jai).

A temporary cluster runs PostgreSQL's regression suite with the real psql, which leaves its tables
(typed data of every built-in type) in the `regression` database. roundtrip.jai then fetches every
column of a type the Postgres client decodes in text and in binary, decodes both, sends the value
back as a parameter and compares it on the server. The same program checks the awkward values of
edge_values.sql and every json/jsonb literal the JSON regression files use
(docs/postgres/conformance.md).

    python3 tools/pg_conformance/roundtrip.py
    python3 tools/pg_conformance/roundtrip.py --only int4 numeric jsonb   # a smaller regression run
    python3 tools/pg_conformance/roundtrip.py --skip-regress              # edge values and literals only
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from fetch import VERSIONS, fetch
from repo_output import output_dir
from run import CONFORMANCE, build_regress_lib, pg_config, run_suite
from toolkit.cli import add_jaic_argument

JSON_FILES = ("json", "jsonb", "jsonpath", "jsonb_jsonpath", "json_encoding", "sqljson*")
JSON_TYPE = re.compile(r"(?:^|[^\w$])(json|jsonb)\s*$", re.IGNORECASE)
JSON_CAST = re.compile(r"\s*::\s*(json|jsonb)\b", re.IGNORECASE)
DOLLAR_TAG = re.compile(r"\$(?:[A-Za-z_\x80-\uffff][\w\x80-\uffff]*)?\$")


def sql_literals(source: str):
    """Yields (text, before, after) for each standard string literal of a SQL file: its contents,
    and the SQL just before and after it. Comments, E'' strings, quoted identifiers and dollar
    quoted bodies are skipped."""
    i, n = 0, len(source)

    while i < n:
        c = source[i]

        if source.startswith("--", i):
            end = source.find("\n", i)
            i = n if end < 0 else end
        elif source.startswith("/*", i):
            depth = 1
            i += 2

            while i < n and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
        elif c == '"':
            end = i + 1

            while end < n and not (source[end] == '"' and source[end + 1 : end + 2] != '"'):
                end += 2 if source[end] == '"' else 1

            i = end + 1
        elif c == "$" and (tag := DOLLAR_TAG.match(source, i)):
            end = source.find(tag.group(0), tag.end())
            i = n if end < 0 else end + len(tag.group(0))
        elif c == "'":
            escaped = i > 0 and source[i - 1] in "eE" and (i < 2 or not re.match(r"[\w$]", source[i - 2]))
            start = i
            i += 1
            parts = []

            while i < n:
                if escaped and source[i] == "\\":
                    i += 2
                elif source[i] == "'":
                    if source[i + 1 : i + 2] == "'":
                        parts.append(source[start + 1 : i + 1])
                        start = i + 1
                        i += 2
                    else:
                        break
                else:
                    i += 1

            parts.append(source[start + 1 : i])
            i += 1

            if not escaped:
                yield "".join(parts), source[max(0, start - 40) : start], source[i : i + 40]
        else:
            i += 1


def json_literals(regress: Path) -> list[tuple[str, str]]:
    """The (type, text) pairs of the JSON regression files: literals cast to json or jsonb, and
    untyped ones that look like a document (a function argument gets its type from the function).
    The server decides which are valid."""
    found: dict[tuple[str, str], None] = {}

    for pattern in JSON_FILES:
        for path in sorted((regress / "sql").glob(pattern + ".sql")):
            for text, before, after in sql_literals(path.read_text(errors="replace")):
                if cast := JSON_CAST.match(after):
                    kind = cast.group(1).lower()
                elif typed := JSON_TYPE.search(before):
                    kind = typed.group(1).lower()
                elif re.match(r"\s*[\[{]", text) and re.search(r"[\]}]\s*$", text):
                    kind = "any"
                else:
                    continue

                found[(kind, text)] = None

    return list(found)


def escape(text: str) -> str:
    return text.replace("\\", "\\\\").replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")


def write_literals(path: Path, literals: list[tuple[str, str]]) -> None:
    path.write_text("".join(f"{kind}\t{escape(text)}\n" for kind, text in literals), encoding="utf-8")


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def start_cluster(config: str, data: Path, socket_dir: Path, port: int, env: dict[str, str]) -> None:
    bindir = Path(pg_config(config, "--bindir"))

    if data.exists():
        shutil.rmtree(data)

    subprocess.run([bindir / "initdb", "-D", data, "-U", "postgres", "-A", "trust", "--no-sync"], check=True, env=env, stdout=subprocess.DEVNULL)

    with open(data / "postgresql.conf", "a") as conf:
        conf.write(f"listen_addresses = ''\nport = {port}\nunix_socket_directories = '{socket_dir}'\nfsync = off\n")

    subprocess.run([bindir / "pg_ctl", "-D", data, "-l", data / "server.log", "-w", "start"], check=True, env=env, stdout=subprocess.DEVNULL)


def stop_cluster(config: str, data: Path, env: dict[str, str]) -> None:
    bindir = Path(pg_config(config, "--bindir"))
    subprocess.run([bindir / "pg_ctl", "-D", data, "-m", "immediate", "stop"], env=env, stdout=subprocess.DEVNULL, check=False)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    add_jaic_argument(parser)
    parser.add_argument("--pg-config", default="pg_config", help="the installed server's pg_config")
    parser.add_argument("--only", nargs="+", help="run these regression tests (after test_setup) instead of the schedule")
    parser.add_argument("--skip-regress", action="store_true", help="skip the regression run: edge values and JSON literals only")
    parser.add_argument("--timezone", default="Europe/Amsterdam", help="the session time zone, for timestamptz text")
    args = parser.parse_args()
    # The build runs in its own folder, so a relative path would point nowhere.
    args.jaic = args.jaic.resolve()

    version = pg_config(args.pg_config, "--version").split()[1]
    major = version.split(".")[0]
    if major not in VERSIONS:
        print(f"no pinned suite for PostgreSQL {version}", file=sys.stderr)
        return 2

    regress = fetch(major) / "src" / "test" / "regress"
    out = output_dir("pg-conformance", "roundtrip", major)
    out.mkdir(parents=True, exist_ok=True)

    env = dict(os.environ)
    # macOS postmasters refuse to start without a valid locale in the environment.
    if not env.get("LC_ALL"):
        for name in ("LC_ALL", "LANG", "LC_CTYPE", "LC_COLLATE"):
            env[name] = "en_US.UTF-8"

    literals = out / "json-literals.txt"
    write_literals(literals, json_literals(regress))
    print(f"{len(literals.read_text().splitlines())} json literals harvested from the regression files")

    build = out / "build"
    build.mkdir(exist_ok=True)
    roundtrip = CONFORMANCE / "roundtrip.jai"
    subprocess.run([str(args.jaic), str(roundtrip), "-exe", "roundtrip"], cwd=build, check=True, env=env)

    # Unix socket paths are short on macOS: the socket lives in a temporary directory.
    socket_dir = Path(tempfile.mkdtemp(prefix="pgrt"))
    data = out / "cluster"
    port = free_port()
    start_cluster(args.pg_config, data, socket_dir, port, env)

    try:
        if not args.skip_regress:
            libdir = build_regress_lib(regress, out, args.pg_config)
            options = argparse.Namespace(pg_config=args.pg_config, port=port, host=str(socket_dir), user="postgres", only=args.only)
            bindir = Path(pg_config(args.pg_config, "--bindir"))
            results = run_suite(options, "regress", bindir, regress, libdir, out)
            print(f"regression run: {sum(results.values())}/{len(results)} tests passed (their tables stay for the round trip)")

        database = "postgres" if args.skip_regress else "regression"
        command = [
            str(build / "roundtrip"),
            "--database", database,
            "--schema", "*",
            "--script", str(CONFORMANCE / "edge_values.sql"),
            "--json-literals", str(literals),
        ]
        env.update(PGHOST=str(socket_dir), PGPORT=str(port), PGUSER="postgres", ROUNDTRIP_TIMEZONE=args.timezone)
        return subprocess.run(command, env=env).returncode
    finally:
        stop_cluster(args.pg_config, data, env)
        shutil.rmtree(socket_dir, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
