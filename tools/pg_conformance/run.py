#!/usr/bin/env python3
"""Runs PostgreSQL's regression suite through the Jai psql shim and through the real psql.

pg_regress drives the server through psql; `--bindir` points it at a directory whose `psql` is
the shim (built from stdlib/Extensions/Postgres/tests/conformance/psql.jai) and whose other
programs are the installed server's. A test counts against the client when it passes with the
real psql and fails with the shim, unless regress-skip.txt lists it (docs/postgres/conformance.md).

    python3 tools/pg_conformance/run.py                    # a temporary cluster per client
    python3 tools/pg_conformance/run.py --port 54329       # an already running server
    python3 tools/pg_conformance/run.py --only boolean int4

    python3 tools/pg_conformance/run.py --tap              # the TAP suites, through the shims
    python3 tools/pg_conformance/run.py --tap --only 001_uri 020_cancel

    python3 tools/pg_conformance/run.py --pipeline-traces  # message traces against libpq_pipeline's
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import signal
import subprocess
import sys
import tarfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from fetch import VERSIONS, fetch
from repo_output import output_dir, shared_output_dir
from toolkit.cli import add_jaic_argument
from toolkit.download import pinned
from toolkit.repo import ROOT

CONFORMANCE = ROOT / "stdlib" / "Extensions" / "Postgres" / "tests" / "conformance"
SKIP_FILE = CONFORMANCE / "regress-skip.txt"

# The TAP files run through the shims, relative to the source tree. A file passes when every test
# in it that passes with the real programs passes with the shims. `real` is false for a file whose
# program PostgreSQL does not install (libpq_uri_regress): its own expectations are the oracle.
# Tests of psql/t/001_basic.pl that need what the shim does not have: name fragment to reason.
TAP_BASIC_SKIPS = {
    "\\help": "the shim has no SQL command help text",
    "errverbose after FETCH_COUNT": "psql before 17 runs a FETCH_COUNT query through a declared cursor, which shifts the error's line; the shim uses chunked mode",
    "PQresultStatus": "replication protocol responses (CopyBoth) are not supported by the client",
}

# Tests of authentication/t/001_password.pl whose expected message is the libpq of their own
# version. When require_auth forbids SCRAM by name, libpq 18 reports "SCRAM-SHA-256"; earlier ones
# report "SASL". The client follows 18, the newest.
TAP_PASSWORD_SKIPS = {
    "forbidden, fails with SCRAM auth": "libpq before 18 words a forbidden SCRAM as SASL authentication",
}

# Tests of ssl/t/002_scram.pl that need what the TLS library behind the client cannot do.
TAP_SSL_SKIPS = {
    "rsassaPss": "mbedTLS reads no certificate whose public key is of the RSASSA-PSS type, and does not offer the rsa_pss_pss signature schemes the server needs for one",
}

TAP_SUITES = (
    {"file": "src/interfaces/libpq/t/001_uri.pl", "real": False},
    {"file": "src/interfaces/libpq/t/003_load_balance_host_list.pl"},
    {"file": "src/test/authentication/t/001_password.pl", "skip_tests": TAP_PASSWORD_SKIPS},
    {"file": "src/test/authentication/t/002_saslprep.pl"},
    {"file": "src/test/authentication/t/003_peer.pl"},
    {"file": "src/test/authentication/t/004_file_inclusion.pl"},
    {"file": "src/test/authentication/t/006_login_trigger.pl"},
    {"file": "src/bin/psql/t/001_basic.pl", "skip_tests": TAP_BASIC_SKIPS},
    {"file": "src/bin/psql/t/020_cancel.pl"},
    # The SSL suites read their certificates relative to their own folder.
    {"file": "src/test/ssl/t/001_ssltests.pl", "cwd": "src/test/ssl"},
    {"file": "src/test/ssl/t/002_scram.pl", "cwd": "src/test/ssl", "skip_tests": TAP_SSL_SKIPS},
)

# TAP files that do not apply, with the reason (docs/postgres/conformance.md).
TAP_SKIPS = {
    "src/test/authentication/t/005_sspi.pl": "Windows only (SSPI)",
    "src/bin/psql/t/010_tab_completion.pl": "drives psql's line editor through a terminal; the shim has none",
    "src/interfaces/libpq/t/002_api.pl": "tests libpq's C API through its own test client",
    "src/interfaces/libpq/t/004_load_balance_dns.pl": "needs one name resolving to several addresses (a hosts file entry), and the client tries the first address only",
    "src/interfaces/libpq/t/005_negotiate_encryption.pl": "the matrix includes GSS encryption rows (Kerberos), which the client does not speak",
}

TAP_TIMEOUT = 900  # seconds for one TAP file

IPC_RUN_VERSION = "IPC-Run-20231003.0"
IPC_RUN_URL = f"https://cpan.metacpan.org/authors/id/T/TO/TODDR/{IPC_RUN_VERSION}.tar.gz"
IPC_RUN_SHA256 = "eb25bbdf5913d291797ef1bfe998f15130b455d3ed02aacde6856f0b25e4fe57"


def pg_config(binary: str, flag: str) -> str:
    return subprocess.run([binary, flag], check=True, capture_output=True, text=True).stdout.strip()


def build_shim(jaic: Path, out: Path, server_bin: Path, prebuilt: Path | None) -> Path:
    """A bin directory holding the shim as `psql`, the URI shim as `libpq_uri_regress` and the
    server's other programs."""
    build = out / "shim-build"
    build.mkdir(parents=True, exist_ok=True)
    if prebuilt:
        shutil.copy2(prebuilt, build / "psql")
    else:
        subprocess.run([str(jaic), str(CONFORMANCE / "psql.jai"), "-exe", "psql"], cwd=build, check=True)
    subprocess.run([str(jaic), str(CONFORMANCE / "uri_regress.jai"), "-exe", "libpq_uri_regress"], cwd=build, check=True)

    bindir = out / "shim-bin"
    if bindir.exists():
        shutil.rmtree(bindir)
    bindir.mkdir()

    for program in server_bin.iterdir():
        if program.name != "psql":
            (bindir / program.name).symlink_to(program.resolve())
    shutil.copy2(build / "psql", bindir / "psql")
    shutil.copy2(build / "libpq_uri_regress", bindir / "libpq_uri_regress")
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
        # macOS servers load ".so" up to PostgreSQL 15 and ".dylib" from 16.
        major = int(pg_config(config, "--version").split()[1].split(".")[0])
        target = libdir / ("regress.so" if major < 16 else "regress.dylib")
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


def ipc_run_lib() -> Path | None:
    """A directory to put on PERL5LIB for IPC::Run, or None when perl has it already. It is
    downloaded from CPAN (pinned by digest) when missing: macOS ships without it."""
    if subprocess.run(["perl", "-MIPC::Run", "-e", "1"], capture_output=True).returncode == 0:
        return None

    target = shared_output_dir("pg-conformance", "ipc-run", IPC_RUN_VERSION)
    lib = target / "lib"
    if (target / ".complete").exists():
        return lib

    archive = pinned(IPC_RUN_URL, IPC_RUN_SHA256, f"perl/{IPC_RUN_VERSION}.tar.gz")
    if target.exists():
        shutil.rmtree(target)
    target.mkdir(parents=True)

    prefix = f"{IPC_RUN_VERSION}/lib/"
    with tarfile.open(archive, "r:gz") as tar:
        members = []
        for member in tar.getmembers():
            if member.name.startswith(prefix):
                member.name = member.name[len(IPC_RUN_VERSION) + 1:]
                members.append(member)
        tar.extractall(target, members=members, filter="data")

    (target / ".complete").write_text("")
    return lib


TAP_LINE = re.compile(r"^(not ok|ok)\s+(\d+)\s*(?:-\s*)?(.*)$")


def parse_tap(text: str) -> dict[int, tuple[bool, str]]:
    """The numbered results of a TAP stream (`prove --verbose` output): number to (passed, name)."""
    results: dict[int, tuple[bool, str]] = {}
    for line in text.splitlines():
        if m := TAP_LINE.match(line):
            results[int(m.group(2))] = (m.group(1) == "ok", m.group(3))
    return results


def run_tap_file(args, suite: dict, source: Path, bindir: Path, mode: str, out: Path, perl_path: list[str]) -> dict[int, tuple[bool, str]]:
    """One TAP file with `bindir` first on PATH; its working files and log stay under `out`."""
    file = source / suite["file"]
    work = out / "tap" / mode / file.stem
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)

    env = dict(os.environ)
    env["PATH"] = os.pathsep.join([str(bindir), env.get("PATH", "")])
    env["PG_REGRESS"] = str(pg_regress(args.pg_config))
    env["TESTDATADIR"] = str(work / "tmp_check")
    env["TESTLOGDIR"] = str(work / "log")
    env["top_builddir"] = str(work)
    # The suites run from the source tree's layout; IPC::Run joins PostgreSQL's own modules.
    env["PERL5LIB"] = os.pathsep.join([*perl_path, env.get("PERL5LIB", "")]).rstrip(os.pathsep)
    # The SSL suites run only when asked for, against a server built with OpenSSL.
    env["PG_TEST_EXTRA"] = " ".join([env.get("PG_TEST_EXTRA", ""), "ssl"]).strip()
    env["with_ssl"] = "openssl"
    # Where the suites read a certificate's serial number from (they only fall back to a pattern without it).
    env.setdefault("OPENSSL", shutil.which("openssl") or "")
    # The test library drops LC_ALL; a macOS postmaster still needs a valid locale from the rest.
    if not env.pop("LC_ALL", ""):
        for name in ("LANG", "LC_CTYPE", "LC_COLLATE"):
            env[name] = "en_US.UTF-8"

    command = ["prove", "--verbose", str(file)]
    print(f"== {mode} {suite['file']}", flush=True)
    # A group of its own, so that a test that hangs takes the servers it started down with it.
    cwd = source / suite["cwd"] if "cwd" in suite else work
    process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace", start_new_session=True)
    try:
        output, _ = process.communicate(timeout=TAP_TIMEOUT)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        output += f"\n# timed out after {TAP_TIMEOUT} seconds\n"
    (work / "prove.log").write_text(output)
    return parse_tap(output)


def run_tap(args) -> int:
    version = pg_config(args.pg_config, "--version").split()[1]
    major = version.split(".")[0]
    if major not in VERSIONS:
        print(f"no pinned suite for PostgreSQL {version}", file=sys.stderr)
        return 2

    source = fetch(major)
    out = output_dir("pg-conformance", "run", major)
    out.mkdir(parents=True, exist_ok=True)

    server_bin = Path(pg_config(args.pg_config, "--bindir"))
    shim_bin = build_shim(args.jaic, out, server_bin, args.shim)
    perl_path = [str(source / "src" / "test" / "perl")]
    ipc = ipc_run_lib()
    if ipc:
        perl_path.append(str(ipc))

    suites = [s for s in TAP_SUITES if not args.only or any(o in s["file"] for o in args.only)]
    for skipped, reason in TAP_SKIPS.items():
        if not args.only or any(o in skipped for o in args.only):
            print(f"skipped {skipped}: {reason}")

    failed = []
    for suite in suites:
        if not (source / suite["file"]).exists():
            print(f"  {suite['file']}: not in PostgreSQL {version}")
            continue

        shim = run_tap_file(args, suite, source, shim_bin, "shim", out, perl_path)
        real: dict[int, tuple[bool, str]] = {}
        if suite.get("real", True) and not args.shim_only:
            real = run_tap_file(args, suite, source, server_bin, "real", out, perl_path)
        elif suite.get("real", True):
            log = out / "tap" / "real" / Path(suite["file"]).stem / "prove.log"
            real = parse_tap(log.read_text(errors="replace")) if log.exists() else {}

        if not shim:
            print(f"  {suite['file']}: no results; see {out / 'tap' / 'shim' / Path(suite['file']).stem / 'prove.log'}")
            failed.append(suite["file"])
            continue

        # With the real programs, only the tests that pass there count; without them (no real
        # program), every test must pass.
        expected = {n for n, (ok, _) in real.items() if ok} if suite.get("real", True) else set(shim)
        skip_tests = suite.get("skip_tests", {})
        skipped = {n: reason for n in expected for fragment, reason in skip_tests.items() if fragment in real.get(n, (True, shim.get(n, (True, "")) [1]))[1] and not shim.get(n, (False, ""))[0]}
        broken = sorted(n for n in expected if not shim.get(n, (False, ""))[0] and n not in skipped)
        passed = sum(1 for ok, _ in shim.values() if ok)
        also = sorted(n for n, (ok, _) in shim.items() if not ok and n not in expected)
        line = f"  {suite['file']}: shim {passed}/{len(shim)} passed"
        if real:
            line += f"; real {sum(1 for ok, _ in real.values() if ok)}/{len(real)}"
        print(line)
        if skipped:
            print(f"    skipped (see the TAP_*_SKIPS tables): {sorted(skipped)}")
        if also:
            print(f"    failing with the real programs too (not counted): {also}")
        if broken:
            names = "; ".join(f"{n} {shim.get(n, (False, 'missing'))[1]}" for n in broken[:8])
            more = f" (and {len(broken) - 8} more)" if len(broken) > 8 else ""
            print(f"    FAILING with the shim only: {names}{more}")
            failed.append(suite["file"])

    if failed:
        print(f"logs: {out / 'tap'}/<shim|real>/<name>/prove.log")
        return 1
    return 0


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
    parser.add_argument("--only", nargs="+", help="run these tests (after test_setup) instead of the schedule; with --tap, the TAP files whose path contains one of these")
    parser.add_argument("--tap", action="store_true", help="run the TAP suites through the shims instead of the regression suite")
    parser.add_argument("--pipeline-traces", action="store_true", help="compare the client's message traces with libpq_pipeline's instead of running the regression suite")
    parser.add_argument("--shim", type=Path, help="a shim binary already built, instead of building one")
    parser.add_argument("--shim-only", action="store_true", help="reuse the last real-psql results")
    args = parser.parse_args()
    # The builds run in their output folders, so a relative path would point nowhere.
    args.jaic = args.jaic.resolve()
    if args.shim:
        args.shim = args.shim.resolve()

    if args.tap:
        return run_tap(args)

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
