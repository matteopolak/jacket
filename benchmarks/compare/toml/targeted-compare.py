"""Targeted warm general-value read/lookup/write comparison, never a replacement for conformance.

Prepare under a normal build lease, run only under an exclusive quiet benchmark lease:
  python3 targeted-compare.py --prepare --root REPO --control CONTROL/build.json --candidate CANDIDATE/build.json --rust ORACLE --rust-receipt ORACLE.json --iterations FROZEN_PLAN.json --workloads INPUTS --output FRESH
  python3 targeted-compare.py --run [same arguments]

All valid read/lookup/write inputs from the frozen plan and all three implementations are
mandatory. Three unscored and twelve scored paired rounds are retained without exclusions.
"""

from pathlib import Path
import argparse
import collections
import hashlib
import json
import statistics
import subprocess
import sys
import tarfile
import tomllib


def writer_probe(sandbox, env, command, expected, equal):
    row = dict(command=command, kind="toml", pass_check=False)
    try:
        run = subprocess.run(
            sandbox.wrap(command), capture_output=True, env=env, timeout=120
        )
        row.update(
            exit=run.returncode,
            output_bytes=len(run.stdout),
            output_checksum=sum(run.stdout),
            output_sha256=hashlib.sha256(run.stdout).hexdigest(),
            stderr_text=run.stderr.decode("utf-8", errors="replace"),
        )
        if run.returncode:
            row["reason"] = "verification exited " + str(run.returncode)
        else:
            row["pass_check"] = equal(
                tomllib.loads(run.stdout.decode("utf-8")), expected
            )
            if not row["pass_check"]:
                row["reason"] = "semantic output mismatch"
    except subprocess.TimeoutExpired:
        row["reason"] = "untimed verification exceeded120seconds"
    except (ValueError, KeyError, TypeError) as error:
        row["reason"] = "invalid verification output: " + type(error).__name__
    return row


def record_sample(run, job, count, fields, journal, samples, apply_metrics):
    row = dict(
        fields,
        stdout_text=run.stdout,
        stderr_text=run.stderr,
        process=dict(
            wall=run.wall, cpu=run.cpu, rss_mib=run.rss_mib, returncode=run.returncode
        ),
    )
    apply_metrics(row, run, job, count)
    samples.append(row)
    with journal.open("a") as stream:
        stream.write(json.dumps(row) + "\n")
    return row


p = argparse.ArgumentParser(description=__doc__)
phase = p.add_mutually_exclusive_group(required=True)
phase.add_argument("--prepare", action="store_true")
phase.add_argument("--run", action="store_true")
for name in [
    "root",
    "control",
    "candidate",
    "rust",
    "rust-receipt",
    "iterations",
    "workloads",
    "output",
]:
    p.add_argument("--" + name, type=Path, required=True)
p.add_argument(
    "--spelling-change",
    action="store_true",
    help="Declare an intentional semantic-writer spelling change; each version still consumes its independently verified bytes.",
)
a = p.parse_args()
if not __debug__:
    raise RuntimeError(
        "Run without Python -O: receipt and inventory checks must remain enabled."
    )
if sys.platform not in ("darwin", "linux"):
    p.error(
        "Targeted benchmark execution currently requires the POSIX tools/bench sandbox."
    )
root = a.root.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=True)
sys.path[:0] = [str(root / "tools"), str(root / "benchmarks/compare/toml")]
import measure as m
from bench.sandbox import Sandbox, environment
from bench.measure import run_measured

sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def authenticate(path):
    d = json.loads(path.read_text())
    binary = path.parent / "jai-batch"
    archive = path.parent / "frozen-owned-sources.tar.gz"
    assert sha(binary) == d["binary_sha256"]
    actual = {}
    with tarfile.open(archive) as t:
        for member in t.getmembers():
            assert member.isfile() and member.name not in actual
            actual[member.name] = hashlib.sha256(
                t.extractfile(member).read()
            ).hexdigest()
    assert actual == d["sources"]
    assert sha(Path(d["command"][0])) == d["compiler_sha256"]
    assert [flag for flag in d["command"] if flag.startswith("-O")] == [
        "-O2"
    ] and "-no_debug_info" in d["command"]
    return dict(
        receipt_sha256=sha(path),
        binary=str(binary),
        binary_sha256=sha(binary),
        archive_sha256=sha(archive),
        compiler_sha256=d["compiler_sha256"],
        community_tree=d.get("community", d.get("community_tree_sha256")),
        sources=d["sources"],
        command=d["command"],
    )


builds = {
    name: authenticate(getattr(a, name).resolve()) for name in ["control", "candidate"]
}
assert builds["control"]["compiler_sha256"] == builds["candidate"]["compiler_sha256"]
assert builds["control"]["community_tree"] == builds["candidate"][
    "community_tree"
] and isinstance(builds["control"]["community_tree"], str)


def computational_command(command):
    result = []
    i = 0
    while i < len(command):
        if command[i] in ["-exe", "-output_path"]:
            i += 2
        else:
            result.append(command[i])
            i += 1
    return result


assert computational_command(builds["control"]["command"]) == computational_command(
    builds["candidate"]["command"]
)
for name in ["batch.jai", "formatter_batch.jai", "settings_projection.jai"]:
    key = "benchmarks/compare/toml/" + name
    assert builds["control"]["sources"][key] == builds["candidate"]["sources"][key]
iteration_plan = json.loads(a.iterations.read_text())
rust_build = json.loads(a.rust_receipt.read_text())
assert (
    sha(a.rust) == rust_build["binary_sha256"] == iteration_plan["rust_binary_sha256"]
)
expected_names = {
    name.rsplit("/", 1)[0]
    for name in iteration_plan["iterations"]
    if name.endswith("/read")
}
expected = {name: iteration_plan["input_sha256"][name] for name in expected_names}
inputs = sorted(a.workloads.resolve().glob("*.toml"))
m.validate_manifest(inputs, dict(input_sha256=expected))
assert all(
    iteration_plan["iterations"][name + "/" + task] > 0
    for name in expected_names
    for task in ["read", "lookup", "write"]
)
protocol_python = {
    name: sha(root / name)
    for name in builds["candidate"]["sources"]
    if name.endswith(".py")
}
configuration = dict(
    protocol_python_sha256=protocol_python,
    controller_sha256=sha(Path(__file__)),
    builds=builds,
    rust=dict(
        binary=str(a.rust.resolve()),
        binary_sha256=sha(a.rust),
        receipt_sha256=sha(a.rust_receipt),
        build=rust_build,
    ),
    iterations_plan_sha256=sha(a.iterations),
    input_sha256=expected,
    iterations={
        name + "/" + task: iteration_plan["iterations"][name + "/" + task]
        for name in expected_names
        for task in ["read", "lookup", "write"]
    },
    spelling_change=a.spelling_change,
    rounds=list(range(-3, 12)),
    scope="All original valid inputs, general-value read/lookup/prepared-write only; no invalid-reject, consumer, competitor or cold tasks in this targeted experiment.",
)
sandbox = Sandbox(
    reads=[
        root,
        *[Path(x["binary"]).parent for x in builds.values()],
        a.rust.resolve().parent,
        a.workloads.resolve(),
    ],
    writes=[out],
)
env = environment(out)


def command(version, task, path, count=1):
    return (
        [str(a.rust.resolve()), "batch-" + task, str(path), str(count)]
        if version == "rust"
        else [builds[version]["binary"], "own-" + task, str(path), str(count)]
    )


if a.prepare:
    assert not (out / "plan.json").exists() and not (out / "validation.json").exists()
    validation = []
    jobs = []
    for path in inputs:
        value = tomllib.loads(path.read_text())
        signatures = {}
        output_hashes = {}
        for version in ["control", "candidate", "rust"]:
            for task in ["verify", "write-verify"]:
                row = (
                    writer_probe(
                        sandbox,
                        env,
                        command(version, task, path),
                        value,
                        m.semantic_equal,
                    )
                    if task == "write-verify"
                    else m.probe(
                        sandbox, env, command(version, task, path), None, "toml", value
                    )[0]
                )
                row.update(input=path.name, version=version, task=task)
                validation.append(row)
                (out / "validation.json").write_text(
                    json.dumps(validation, indent=2) + "\n"
                )
                assert row["pass_check"], row
                if task == "write-verify":
                    signatures[version] = (row["output_bytes"], row["output_checksum"])
                    output_hashes[version] = row["output_sha256"]
            raw = subprocess.run(
                sandbox.wrap(command(version, "lookup", path)),
                env=env,
                capture_output=True,
                check=True,
                timeout=120,
            )
            metrics = json.loads(raw.stdout)
            ok = (
                metrics["iterations"] == 1
                and metrics["checksum"] == len(value)
                and metrics["output_bytes"] == 0
            )
            validation.append(
                dict(
                    input=path.name,
                    version=version,
                    task="lookup",
                    pass_check=ok,
                    metrics=metrics,
                )
            )
            (out / "validation.json").write_text(
                json.dumps(validation, indent=2) + "\n"
            )
            assert ok
            for task in ["read", "lookup", "write"]:
                jobs.append(
                    dict(
                        input=path.name,
                        version=version,
                        task=task,
                        signature=signatures[version]
                        if task == "write"
                        else (0, len(value))
                        if task == "lookup"
                        else (0, 0),
                        checksum=len(value) if task == "lookup" else None,
                    )
                )
        assert (
            a.spelling_change or output_hashes["control"] == output_hashes["candidate"]
        ), "Unexpected writer spelling/order change"
    assert len(validation) == len(inputs) * 9 and all(
        x["pass_check"] for x in validation
    )
    plan = dict(
        configuration=configuration,
        validation_sha256=sha(out / "validation.json"),
        validation=validation,
        jobs=jobs,
        policy="Same family counts, per-input rotated/reversed pairing,3unscored+12scored rounds; all results retained; internal operation clocks only. Jai uses the same pooled harness with explicit free_toml, Rust normal owned teardown. Process-wall polling is coarse and not used for startup claims.",
    )
    (out / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    print(
        "Prepared",
        len(jobs),
        "groups/",
        len(validation),
        "actual-executable checks; frozen counts.",
        flush=True,
    )
    raise SystemExit(0)
plan = json.loads((out / "plan.json").read_text())
assert (
    plan["configuration"] == configuration
    and sha(out / "validation.json") == plan["validation_sha256"]
)
assert all(x["pass_check"] for x in plan["validation"])
assert not (out / "results.json").exists() and not (out / "samples.jsonl").exists()
report = dict(
    plan_sha256=sha(out / "plan.json"),
    configuration=configuration,
    clocks=m.clock_facts(),
    machine=m.machine_info(),
    python=sys.version,
    samples=[],
    summaries={},
    paired={},
    complete=False,
)
for path in inputs:
    jobs = [j for j in plan["jobs"] if j["input"] == path.name]
    for round_number in range(-3, 12):
        offset = round_number % len(jobs)
        ordered = jobs[offset:] + jobs[:offset]
        if round_number % 2:
            ordered = list(reversed(ordered))
        for spec in ordered:
            count = configuration["iterations"][path.name + "/" + spec["task"]]
            cmd = command(spec["version"], spec["task"], path, count)
            job = m.Job(
                spec["version"] + "-" + spec["task"],
                cmd,
                None,
                signature=tuple(spec["signature"]) if spec["signature"] else None,
                checksum=spec["checksum"],
            )
            run = run_measured(sandbox.wrap(cmd), env=env, timeout=300)
            row = record_sample(
                run,
                job,
                count,
                dict(
                    input=path.name,
                    input_sha256=sha(path),
                    task=job.name,
                    phase="warm",
                    round=round_number,
                    warmup=round_number < 0,
                    command=cmd,
                ),
                out / "samples.jsonl",
                report["samples"],
                m.apply_metrics,
            )
            if "failure" in row:
                (out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
                raise RuntimeError("Failed row retained: " + job.name + "/" + path.name)
    print(path.name, "all paired rounds retained", flush=True)
    (out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
groups = collections.defaultdict(list)
for row in report["samples"]:
    groups[row["input"] + "/" + row["task"]].append(row)
expected_groups = {
    j["input"] + "/" + j["version"] + "-" + j["task"] for j in plan["jobs"]
}
assert (
    set(groups) == expected_groups
    and len(report["samples"]) == len(expected_groups) * 15
)
for key, rows in groups.items():
    assert collections.Counter(r["round"] for r in rows) == collections.Counter(
        range(-3, 12)
    )
    xs = [r["ns_per_iteration"] for r in rows if not r["warmup"]]
    assert len(xs) == 12
    report["summaries"][key] = dict(
        min=min(xs), median=statistics.median(xs), max=max(xs)
    )
for path in inputs:
    for task in ["read", "lookup", "write"]:
        cand = {
            r["round"]: r["ns_per_iteration"]
            for r in groups[path.name + "/candidate-" + task]
            if not r["warmup"]
        }
        for other in ["control", "rust"]:
            baseline = {
                r["round"]: r["ns_per_iteration"]
                for r in groups[path.name + "/" + other + "-" + task]
                if not r["warmup"]
            }
            xs = [cand[n] / baseline[n] for n in range(12)]
            report["paired"][path.name + "/" + task + "/candidate-over-" + other] = (
                dict(min=min(xs), median=statistics.median(xs), max=max(xs))
            )
assert configuration == dict(
    configuration,
    builds={
        name: authenticate(getattr(a, name).resolve())
        for name in ["control", "candidate"]
    },
)
assert sha(a.rust) == configuration["rust"]["binary_sha256"]
assert protocol_python == {name: sha(root / name) for name in protocol_python}
m.validate_manifest(
    sorted(a.workloads.resolve().glob("*.toml")), dict(input_sha256=expected)
)
assert (
    sha(Path(__file__)) == configuration["controller_sha256"]
    and sha(out / "plan.json") == report["plan_sha256"]
)
report["complete"] = True
(out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(
    "Targeted warm comparison complete:",
    len(report["samples"]),
    "rows/",
    len(groups),
    "groups; no exclusions.",
    flush=True,
)
