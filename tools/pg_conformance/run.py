#!/usr/bin/env python3
"""Runs PostgreSQL's regression suite through the Jai psql shim and through the real psql.

pg_regress drives the server through psql; `--bindir` points it at a directory whose `psql` is
the shim (built from stdlib/Extensions/Postgres/tests/conformance/psql.jai) and whose other
programs are the installed server's. A test counts against the client when it passes with the
real psql and fails with the shim, unless regress-skip.txt lists it (docs/postgres/conformance.md).

    python3 tools/pg_conformance/run.py                    # a temporary cluster per client
    python3 tools/pg_conformance/run.py --port 54329       # an already running server
    python3 tools/pg_conformance/run.py --only boolean int4

    python3 tools/pg_conformance/run.py --pipeline-traces  # message traces against libpq_pipeline's
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from fetch import VERSIONS, fetch
from repo_output import output_dir
from toolkit.cli import add_jaic_argument
from toolkit.repo import ROOT

CONFORMANCE = ROOT / "stdlib" / "Extensions" / "Postgres" / "tests" / "conformance"
SKIP_FILE = CONFORMANCE / "regress-skip.txt"


def pg_config(binary: str, flag: str) -> str:
    return subprocess.run([binary, flag], check=True, capture_output=True, text=True).stdout.strip()


def build_shim(jaic: Path, out: Path, server_bin: Path, prebuilt: Path | None) -> Path:
    """A bin directory holding the shim as `psql` and the server's other programs."""
    build = out / "shim-build"
    build.mkdir(parents=True, exist_ok=True)
    if prebuilt:
        shutil.copy2(prebuilt, build / "psql")
    else:
        subprocess.run([str(jaic), str(CONFORMANCE / "psql.jai"), "-exe", "psql"], cwd=build, check=True)

    bindir = out / "shim-bin"
    if bindir.exists():
        shutil.rmtree(bindir)
    bindir.mkdir()

    for program in server_bin.iterdir():
        if program.name != "psql":
            (bindir / program.name).symlink_to(program.resolve())
    shutil.copy2(build / "psql", bindir / "psql")
    return bindir


def build_regress_lib(source: Path, out: Path, config: str) -> Path:
    """regress.c, which the suite loads for its C test functions, and contrib/spi's autoinc and
    refint for the triggers test."""
    libdir = out / "lib"
    libdir.mkdir(parents=True, exist_ok=True)
    includes = ["-I" + pg_config(config, "--includedir-server")]

    if sys.platform == "darwin":
        for extra in ("/opt/homebrew/include", "/opt/homebrew/opt/gettext/include", "/usr/local/include"):
            if Path(extra).is_dir():
                includes.append("-I" + extra)
        postgres = Path(pg_config(config, "--bindir")) / "postgres"
        link = ["-bundle", "-bundle_loader", str(postgres)]
        target = libdir / "regress.dylib"
    else:
        link = ["-shared"]
        target = libdir / "regress.so"

    spi = source.parents[2] / "contrib" / "spi"
    # refint's NOTICEs are part of the expected output; contrib/spi's Makefile turns them on.
    for c_file, name, defines in (
        (source / "regress.c", "regress", []),
        (spi / "autoinc.c", "autoinc", []),
        (spi / "refint.c", "refint", ["-DREFINT_VERBOSE"]),
    ):
        library = target.with_name(name + target.suffix)
        command = ["cc", "-O2", "-fPIC", "-w", *defines, *includes, *link, "-o", str(library), str(c_file)]
        subprocess.run(command, check=True)
    return libdir


def pg_regress(config: str) -> Path:
    return Path(pg_config(config, "--pkglibdir")) / "pgxs" / "src" / "test" / "regress" / "pg_regress"


def run_suite(args, name: str, bindir: Path, regress: Path, libdir: Path, out: Path) -> dict[str, bool]:
    outdir = out / name
    if outdir.exists():
        shutil.rmtree(outdir)
    outdir.mkdir(parents=True)

    command = [
        str(pg_regress(args.pg_config)),
        f"--inputdir={regress}",
        f"--outputdir={outdir}",
        f"--bindir={bindir}",
        f"--dlpath={libdir}",
        "--max-connections=8",
    ]

    if args.port:
        command += [f"--host={args.host}", f"--port={args.port}", f"--user={args.user}"]
    else:
        command += [f"--temp-instance={outdir / 'tmp_check'}"]

    if args.only:
        command += ["test_setup", *[t for t in args.only if t != "test_setup"]]
    else:
        command += [f"--schedule={regress / 'parallel_schedule'}"]

    env = dict(os.environ)
    # macOS postmasters refuse to start without a valid locale in the environment.
    # pg_regress clears LC_ALL, so the individual categories are set as well.
    if not env.get("LC_ALL"):
        for name in ("LC_ALL", "LANG", "LC_CTYPE", "LC_COLLATE"):
            env[name] = "en_US.UTF-8"
    if args.port:
        env["PGUSER"] = args.user

    print(f"== {name}: {' '.join(command)}", flush=True)
    # pg_regress deletes regression.out when every test passes, so the results come from its
    # output, kept in summary.txt.
    run = subprocess.run(command, cwd=outdir, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    print(run.stdout, end="", flush=True)
    summary = outdir / "summary.txt"
    summary.write_text(run.stdout)
    return parse_results(summary)


RESULT_TAP = re.compile(r"^(not ok|ok)\s+\d+\s+[-+]\s+(\S+)")
RESULT_OLD = re.compile(r"^\s*(?:test\s+)?(\S+)\s+\.\.\.\s+(ok|FAILED|failed \(ignored\))")


def parse_results(path: Path) -> dict[str, bool]:
    results: dict[str, bool] = {}
    if not path.exists():
        return results

    for line in path.read_text(errors="replace").splitlines():
        if m := RESULT_TAP.match(line):
            results[m.group(2)] = m.group(1) == "ok"
        elif m := RESULT_OLD.match(line):
            results[m.group(1)] = m.group(2) == "ok"
    return results


def read_skips() -> dict[str, str]:
    skips: dict[str, str] = {}
    if not SKIP_FILE.exists():
        return skips

    for line in SKIP_FILE.read_text().splitlines():
        text = line.split("#", 1)
        name = text[0].strip()
        if name:
            skips[name] = text[1].strip() if len(text) > 1 else ""
    return skips


def run_pipeline_traces(args, major: str) -> int:
    """Replays libpq_pipeline's scenarios on our client and compares its message trace with the
    .trace files libpq wrote (tests/conformance/pipeline_traces.jai). Uses the server at --port,
    or starts a temporary one."""
    import socket
    import tempfile

    if int(major) < 17:
        # Older libpq_pipeline scenarios differ (no chunked rows, no close of prepared statements).
        print(f"pipeline traces: skipped for PostgreSQL {major}")
        return 0

    traces = fetch(major) / "src" / "test" / "modules" / "libpq_pipeline" / "traces"
    out = output_dir("pg-conformance", "run", major) / "pipeline-traces"
    out.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(args.jaic), str(CONFORMANCE / "pipeline_traces.jai"), "-exe", "pipeline_traces"], cwd=out, check=True)
    program = str(out / "pipeline_traces")
    env = dict(os.environ, PG_PIPELINE_TRACES=str(traces), PGUSER=args.user)

    if args.port:
        env.update(PGHOST=args.host, PGPORT=str(args.port))
        return subprocess.run([program], env=env).returncode

    server_bin = Path(pg_config(args.pg_config, "--bindir"))
    for name in ("LC_ALL", "LANG", "LC_CTYPE", "LC_COLLATE"):
        env.setdefault(name, "en_US.UTF-8")

    with tempfile.TemporaryDirectory(prefix="pgtr") as tmp:
        data = Path(tmp) / "data"
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]

        initdb = [str(server_bin / "initdb"), "-D", str(data), "-A", "trust", "-U", "postgres", "--no-sync"]
        subprocess.run(initdb, env=env, check=True, stdout=subprocess.DEVNULL)
        options = f"-p {port} -k {tmp} -c listen_addresses=''"
        ctl = [str(server_bin / "pg_ctl"), "-D", str(data), "-l", str(Path(tmp) / "log")]
        subprocess.run([*ctl, "-o", options, "-w", "start"], env=env, check=True, stdout=subprocess.DEVNULL)
        try:
            env.update(PGHOST=tmp, PGPORT=str(port), PGUSER="postgres")
            return subprocess.run([program], env=env).returncode
        finally:
            subprocess.run([*ctl, "-m", "fast", "stop"], env=env, stdout=subprocess.DEVNULL)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    add_jaic_argument(parser)
    parser.add_argument("--pg-config", default="pg_config", help="the installed server's pg_config")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, help="use this running server instead of temporary clusters")
    parser.add_argument("--user", default="postgres", help="the user to connect as, with --port")
    parser.add_argument("--only", nargs="+", help="run these tests (after test_setup) instead of the schedule")
    parser.add_argument("--pipeline-traces", action="store_true", help="compare the client's message traces with libpq_pipeline's instead of running the regression suite")
    parser.add_argument("--shim", type=Path, help="a shim binary already built, instead of building one")
    parser.add_argument("--shim-only", action="store_true", help="reuse the last real-psql results")
    args = parser.parse_args()

    version = pg_config(args.pg_config, "--version").split()[1]
    major = version.split(".")[0]
    if major not in VERSIONS:
        print(f"no pinned suite for PostgreSQL {version}", file=sys.stderr)
        return 2

    if args.pipeline_traces:
        return run_pipeline_traces(args, major)

    regress = fetch(major) / "src" / "test" / "regress"
    out = output_dir("pg-conformance", "run", major)
    out.mkdir(parents=True, exist_ok=True)

    server_bin = Path(pg_config(args.pg_config, "--bindir"))
    libdir = build_regress_lib(regress, out, args.pg_config)
    shim_bin = build_shim(args.jaic, out, server_bin, args.shim)

    if args.shim_only:
        real = parse_results(out / "real" / "summary.txt")
    else:
        real = run_suite(args, "real", server_bin, regress, libdir, out)
    shim = run_suite(args, "shim", shim_bin, regress, libdir, out)
    skips = read_skips()

    if not shim or not real:
        print("a run produced no results; see its log under " + str(out), file=sys.stderr)
        return 1

    regressions = [t for t, ok in shim.items() if not ok and real.get(t) and t not in skips]
    both = [t for t, ok in shim.items() if not ok and not real.get(t, True)]
    skipped = [t for t, ok in shim.items() if not ok and t in skips]
    fixed = [t for t in skips if shim.get(t)]

    passed = sum(1 for ok in shim.values() if ok)
    print(f"\nshim: {passed}/{len(shim)} passed; real psql: {sum(real.values())}/{len(real)}")
    if both:
        print(f"failing with real psql too (not counted): {' '.join(both)}")
    if skipped:
        print(f"failing, in regress-skip.txt: {' '.join(skipped)}")
    if fixed:
        print(f"passing now, remove from regress-skip.txt: {' '.join(fixed)}")
    if regressions:
        print(f"FAILING with the shim only: {' '.join(regressions)}")
        print(f"diffs: {out / 'shim' / 'regression.diffs'}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
