#!/usr/bin/env python3
"""The files each CI job reads, computed from the workflow instead of listed by hand.

    python3 tools/ci/job_inputs.py --print [job ...]     # the input patterns of the gated jobs

`inputs(job)` is the union of
  * the repo paths a job's steps name (`run:`, `with:`, the composite actions it uses, recursively),
  * the closure of the scripts among them (Python imports and path strings, shell `source` and
    paths, PowerShell and Nix paths),
  * `crates/<name>/**` for the crates the job builds (CRATES) and their path dependencies, from
    the `Cargo.toml` files, plus the lock and toolchain files,
  * the Jai sources the `.jai` programs among them read: `jaic -deps` when a compiler is at hand
    (`$JAIC_DEPS_JAIC`), otherwise a scan of their `#import`/`#load` lines, plus the prelude,
  * EXTRA, the few inputs no step names.

Patterns are paths or globs (`*` within a directory, `**` across them). A directory is `dir/**`.
Paths a script builds at run time (`f"tools/{name}.py"`) are found only up to their literal prefix.
Adding a step, a script, an import or a path dependency needs no edit here; a job that builds a
different binary needs a row in CRATES (`drift` fails the tests when a `cargo -p` names a crate the
row lacks).
"""

import ast
import bisect
import fnmatch
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ".github/workflows/ci.yml"

# The jobs gated on their inputs (the others run whenever code changed).
GATED = {
    "sdl3",
    "clipboard-linux",
    "nix",
    "shellcheck",
    "install-sh",
    "install-ps1",
    "homebrew",
    "sanitizers",
    "windows-cross",
    "windows-run-cross",
    "windows-native",
    "windows-stdlib",
    "windows-smoke",
}

# The packages each job builds with `cargo -p` (or through Nix). Their path dependencies follow.
CRATES = {
    "sdl3": ["jaic-cli"],
    "clipboard-linux": ["jaic-cli", "jailint"],
    "nix": ["jaic-cli", "jailint", "jai-language-server"],
    "sanitizers": ["jaic-cli", "jacket"],
    "windows-cross": ["jaic-cli"],
    "windows-native": ["jaic-cli", "jailint"],
    "windows-stdlib": ["jaic-cli", "jacket"],
    "windows-smoke": ["jaic-cli"],
}
# Files every job that builds a crate reads.
CARGO_FILES = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]

# Inputs no step names: what a tool reads by its own rule. Keep this short.
EXTRA = {
    # shellcheck lints every shell script, found by `tools/dev/staged-files.sh`.
    "shellcheck": ["**.sh", ".githooks/**"],
    # render_packages.py reads its templates through a `ROOT / "packaging"` variable.
    "homebrew": ["packaging/**"],
    # The native test reads every program under tests/compiler/native.
    "windows-native": ["tests/compiler/native/**"],
    # `nix flake check` evaluates the flake, whose files `nix/` imports.
    "nix": ["flake.nix", "flake.lock"],
}

# These build and run whatever the standard library holds, so any change to it selects them.
WHOLE_STDLIB = (
    "sanitizers",
    "windows-cross",
    "windows-run-cross",
    "windows-native",
    "windows-stdlib",
    "windows-smoke",
)
for _job in WHOLE_STDLIB:
    EXTRA[_job] = EXTRA.get(_job, []) + ["stdlib/**"]

# The test runner discovers its tests by name; the folders and patterns it is given stand for them.
RUNNER = "tools/run_tests.py"

SKIP_DIRS = {".git", "target", "node_modules"}
# Prefixes of paths that exist only after a build or a fetch.
GENERATED = ("corpus/upstream",)

PATH_CHARS = r"[\w./*@+-]+"
GLOB = re.compile(r"[*?\[]")


class Repo:
    """The tracked files under `root`, and what top-level names a path can start with."""

    def __init__(self, root=ROOT, files=None):
        self.root = Path(root)
        self.files = sorted(files if files is not None else self._list())
        self.fileset = set(self.files)
        self.dirs = {f.rsplit("/", 1)[0] for f in self.files if "/" in f}
        for d in list(self.dirs):
            while "/" in d:
                d = d.rsplit("/", 1)[0]
                self.dirs.add(d)
        self.top_dirs = sorted({f.split("/", 1)[0] for f in self.files if "/" in f})
        self.top_files = sorted(f for f in self.files if "/" not in f)
        names = [re.escape(d) for d in self.top_dirs]
        top = rf"(?:{'|'.join(names)})/{PATH_CHARS}" if names else r"(?!)"
        files_rx = "|".join(re.escape(f) for f in self.top_files)
        alt = top + (rf"|(?:{files_rx})(?![\w/-])" if files_rx else "")
        self.path_rx = re.compile(rf"(?<![\w./$-])(?:{alt})")
        self._memo = {}
        self._texts = {}

    def _list(self):
        if (self.root / ".git").exists():
            out = subprocess.run(
                ["git", "-c", "core.quotepath=off", "ls-files", "-z"],
                cwd=self.root,
                capture_output=True,
                text=True,
            )
            if out.returncode == 0:
                return [
                    f for f in out.stdout.split("\0") if f and (self.root / f).exists()
                ]
        found = []
        for base, dirs, names in os.walk(self.root):
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            rel = Path(base).relative_to(self.root)
            found += [(rel / n).as_posix() for n in names]
        return found

    def read(self, path):
        if path not in self._texts:
            try:
                self._texts[path] = (self.root / path).read_text(errors="replace")
            except OSError:
                self._texts[path] = ""
        return self._texts[path]

    def under(self, directory):
        prefix = directory.rstrip("/") + "/"
        low = bisect.bisect_left(self.files, prefix)
        high = bisect.bisect_left(
            self.files, prefix[:-1] + "0"
        )  # "0" is the character after "/"
        return self.files[low:high]


def runner_args(repo, text):
    """The test files `tools/run_tests.py` is asked to run: `-p` patterns and folder names."""
    out = set()
    for args in re.findall(r"tools/run_tests\.py([^\n|;&]*)", normalize(text)):
        tokens = args.replace("'", " ").replace('"', " ").split()
        skip = False
        for i, token in enumerate(tokens):
            if skip:
                skip = False
            elif token in ("-p", "--pattern") and i + 1 < len(tokens):
                value = tokens[i + 1]
                out.add(value if "/" in value else "tools/**/" + value)
                skip = True
            elif token in ("-j", "--timeout", "--jobs"):
                skip = True
            elif not token.startswith("-") and f"tools/{token}" in repo.dirs:
                out.add(f"tools/{token}/**")
    for folder, name in re.findall(
        r"unittest discover\s+-s\s+(\S+)(?:[^\n|;&]*?\s-p\s+(\S+))?",
        normalize(text, True),
    ):
        out.add(f"{folder.strip(chr(39) + chr(34))}/{name or 'test*.py'}")
    return out


def normalize(text, windows=False):
    """Windows separators (when `windows`) and the checkout prefixes the steps use become plain
    repo-relative paths."""
    if windows:
        text = text.replace("\\", "/")
    for prefix in (
        "${{ github.workspace }}/",
        "$GITHUB_WORKSPACE/",
        "${GITHUB_WORKSPACE}/",
        "$PWD/",
        "${PWD}/",
    ):
        text = text.replace(prefix, "")
    return re.sub(r"(?<![\w.])\./", "", text)


def paths_in(repo, text, windows=False):
    """The repo paths and globs `text` names, as patterns (a directory becomes `dir/**`)."""
    out = set()
    # `unittest discover -s DIR -p FILE` names FILE, not all of DIR (runner_args).
    text = re.sub(r"unittest discover\s+-s\s+\S+", "unittest discover", text)
    for match in repo.path_rx.findall(normalize(text, windows)):
        path = match.rstrip(".,:;)'\"`").rstrip("/")
        if not path or any(path == g or path.startswith(g + "/") for g in GENERATED):
            continue
        out.add(pattern(repo, path))
    return out


def pattern(repo, path):
    if GLOB.search(path):
        return path
    if path in repo.dirs:
        return path + "/**"
    return path


def sibling_scripts(repo, directory, text):
    """Bare `name.sh`/`name.py` mentions that are files next to the script."""
    out = set()
    for name in re.findall(r"[\w-]+\.(?:sh|py|ps1)\b", text):
        candidate = f"{directory}/{name}" if directory else name
        if candidate in repo.fileset:
            out.add(candidate)
    return out


def const_runs(tree):
    """The path text of `a / "x" / "y"`, `os.path.join(a, "x", "y")` and `Path("x", "y")` runs."""
    runs = []

    def flatten(node):
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            return flatten(node.left) + flatten(node.right)
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return [node.value]
        return [None]

    def emit(parts):
        run = []
        for part in parts + [None]:
            if part is None:
                if len(run) > 1:
                    runs.append("/".join(run))
                run = []
            else:
                run.append(part.strip("/"))

    for node in ast.walk(tree):
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            emit(flatten(node))
        elif isinstance(node, ast.Call):
            name = (
                node.func.attr
                if isinstance(node.func, ast.Attribute)
                else getattr(node.func, "id", "")
            )
            if name in ("join", "Path", "PurePath", "joinpath"):
                emit(
                    [
                        a.value
                        if isinstance(a, ast.Constant) and isinstance(a.value, str)
                        else None
                        for a in node.args
                    ]
                )
    return runs


def python_refs(repo, path):
    """Files a Python script imports, and the repo paths its string literals name."""
    text = repo.read(path)
    try:
        tree = ast.parse(text)
    except SyntaxError:
        return paths_in(repo, text)
    here = path.rsplit("/", 1)[0] if "/" in path else ""
    roots = [here, "tools", ""]

    def module(dotted):
        rel = dotted.replace(".", "/")
        for base in roots:
            for candidate in (f"{rel}.py", f"{rel}/__init__.py"):
                full = f"{base}/{candidate}" if base else candidate
                if full in repo.fileset:
                    return full
        return None

    docstrings = {
        id(n.body[0].value)
        for n in ast.walk(tree)
        if isinstance(
            n, (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)
        )
        and n.body
        and isinstance(n.body[0], ast.Expr)
        and isinstance(n.body[0].value, ast.Constant)
    }
    found, strings = set(), []
    for node in ast.walk(tree):
        names = []
        if isinstance(node, ast.Import):
            names = [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
            names = [node.module] + [
                f"{node.module}.{alias.name}" for alias in node.names
            ]
        elif (
            isinstance(node, ast.Constant)
            and isinstance(node.value, str)
            and id(node) not in docstrings
        ):
            strings.append(node.value)
        for name in names:
            hit = module(name)
            if hit:
                found.add(hit)
    for value in strings + const_runs(tree):
        found |= paths_in(repo, value)
        if "/" not in value:
            found |= sibling_scripts(repo, here, value)
            # A module named by a string (`("check_x", ...)` run with importlib).
            if re.fullmatch(r"\w+", value) and f"{here}/{value}.py" in repo.fileset:
                found.add(f"{here}/{value}.py")
    return found


def text_refs(repo, path):
    """Files a shell, PowerShell or Nix file names: repo paths, siblings, and `./x` of a Nix file."""
    text = strip_comments(repo.read(path))
    here = path.rsplit("/", 1)[0] if "/" in path else ""
    found = (
        paths_in(repo, text, path.endswith(".ps1"))
        | sibling_scripts(repo, here, text)
        | runner_args(repo, text)
    )
    if path.endswith(".nix"):
        for rel in re.findall(r"(?<![\w$])(\.{1,2}/[\w./-]+)", text):
            full = os.path.normpath(os.path.join(here, rel)).replace(os.sep, "/")
            if full in repo.fileset or full in repo.dirs:
                found.add(pattern(repo, full))
    return found


def jai_refs(repo, path, jaic=None):
    """The Jai files a program reads (`jaic -deps`, or its `#import`/`#load` lines), and the prelude."""
    key = (path, jaic)
    if key not in repo._memo:
        repo._memo[key] = _jai_refs(repo, path, jaic)
    return repo._memo[key]


def _jai_refs(repo, path, jaic):
    if jaic:
        run = subprocess.run(
            [jaic, "-deps", str(repo.root / path)], capture_output=True, text=True
        )
        if run.returncode == 0:
            out = set()
            for line in run.stdout.split():
                try:
                    out.add(
                        Path(line).resolve().relative_to(repo.root.resolve()).as_posix()
                    )
                except ValueError:
                    pass
            return out
    here = path.rsplit("/", 1)[0] if "/" in path else ""
    found = set(prelude(repo))
    for kind, name in re.findall(
        r'#(import(?:,\w+)*|load)\s*(?:,\w+\s*)*"([^"]+)"', repo.read(path)
    ):
        found.update(resolve_jai(repo, here, kind, name))
    return found


def prelude(repo):
    if "prelude" not in repo._memo:
        repo._memo["prelude"] = repo.under("stdlib/prelude")
    return repo._memo["prelude"]


def resolve_jai(repo, here, kind, name):
    def rel(p):
        return os.path.normpath(os.path.join(here, p)).replace(os.sep, "/")

    if kind == "load" or "file" in kind:
        return [rel(name)]
    if "string" in kind:
        return []
    if "dir" in kind:
        return [rel(name) + "/module.jai"]
    options = [
        f"stdlib/{name}/module.jai",
        f"stdlib/{name}.jai",
        rel(name) + "/module.jai",
        rel(name) + ".jai",
    ]
    return [o for o in options if o in repo.fileset][:1]


def closure(repo, seeds, jaic=None, via=None):
    """Every pattern reachable from `seeds` through scripts and Jai programs; `via`, when given,
    records which file first named each one."""
    result, todo = set(), [(s, None) for s in seeds]
    while todo:
        item, parent = todo.pop()
        if item in result:
            continue
        result.add(item)
        if via is not None and parent:
            via[item] = parent
        if (
            item.startswith("stdlib/")
            and item != "stdlib/**"
            and item.endswith("/**")
            and item[:-3] in repo.dirs
        ):
            # A stdlib folder's programs read other modules.
            todo += [(f, item) for f in repo.under(item[:-3]) if f.endswith(".jai")]
            continue
        if GLOB.search(item) or item not in repo.fileset or item == RUNNER:
            continue
        if item.endswith(".py"):
            found = python_refs(repo, item)
        elif item.endswith((".sh", ".ps1", ".nix", ".bash")):
            found = text_refs(repo, item)
        elif item.endswith(".jai"):
            found = jai_refs(repo, item, jaic)
        else:
            continue
        todo += [(f, item) for f in found]
    return result


def jobs_of(workflow_text):
    """{job id: its lines without comments} from the `jobs:` section."""
    body = workflow_text.split("\njobs:\n", 1)[-1]
    jobs = {}
    for block in re.split(r"\n(?=  [A-Za-z0-9_-]+:[ \t]*(?:#.*)?\n)", "\n" + body):
        head = re.match(r"\s*([A-Za-z0-9_-]+):", block)
        if head:
            jobs[head.group(1)] = strip_comments(block)
    return jobs


def strip_comments(text):
    return "\n".join(
        line for line in text.splitlines() if not line.lstrip().startswith("#")
    )


def actions_of(text):
    return re.findall(r"uses:\s*\./(\.github/actions/[\w-]+)", text)


def step_text(repo, text, seen=None):
    """`text` plus the text of every composite action it uses, recursively."""
    seen = seen if seen is not None else set()
    out = [text]
    for action in actions_of(text):
        if action in seen:
            continue
        seen.add(action)
        for name in ("action.yml", "action.yaml"):
            body = repo.read(f"{action}/{name}")
            if body:
                out.append(step_text(repo, strip_comments(body), seen))
    return "\n".join(out)


def cargo_packages(text):
    """The `-p`/`--package` crates of the `cargo` commands in `text`."""
    found = set()
    for command in re.findall(r"\bcargo\b[^\n;|&]*", text):
        found |= set(re.findall(r"(?:^|\s)(?:-p|--package)[ =]([\w-]+)", command))
        found |= set(re.findall(r"(?:^|\s)-p\s*([\w-]+)", command))
    return found


def drift(workflow_text, crates=None, only=None):
    """Problems: a `cargo -p X` in a job's steps that the job's CRATES row does not list."""
    crates = CRATES if crates is None else crates
    problems = []
    for job, text in jobs_of(workflow_text).items():
        if only is not None and job not in only:
            continue
        for name in sorted(cargo_packages(text) - set(crates.get(job, []))):
            problems.append(f"{job}: cargo -p {name} is not in CRATES[{job!r}]")
    return problems


def crate_graph(repo):
    """{package: (directory, [path dependencies])} for the workspace members, read from the
    Cargo.toml files (no `cargo`, which would install the pinned toolchain in a bare checkout)."""
    root = tomllib.loads(repo.read("Cargo.toml"))
    shared = root.get("workspace", {}).get("dependencies", {})
    members = []
    for member in root.get("workspace", {}).get("members", []):
        if "*" in member:
            members += sorted(
                {
                    f.rsplit("/", 1)[0]
                    for f in repo.files
                    if fnmatch.fnmatch(f, member + "/Cargo.toml")
                }
            )
        else:
            members.append(member)
    graph, paths = {}, {}
    for directory in members:
        manifest = tomllib.loads(repo.read(f"{directory}/Cargo.toml"))
        name = manifest.get("package", {}).get("name")
        if not name:
            continue
        tables = [
            manifest.get(k, {})
            for k in ("dependencies", "dev-dependencies", "build-dependencies")
        ]
        for target in manifest.get("target", {}).values():
            tables += [
                target.get(k, {})
                for k in ("dependencies", "dev-dependencies", "build-dependencies")
            ]
        deps = []
        for table in tables:
            for dep, spec in table.items():
                base = directory
                if isinstance(spec, dict) and spec.get("workspace"):
                    spec, base = (
                        shared.get(dep, {}),
                        "",
                    )  # the root manifest's paths are root-relative
                if isinstance(spec, dict) and "path" in spec:
                    deps.append(
                        os.path.normpath(os.path.join(base, spec["path"])).replace(
                            os.sep, "/"
                        )
                    )
        graph[name] = (directory, deps)
        paths[directory] = name
    return {
        name: (d, [paths[x] for x in deps if x in paths])
        for name, (d, deps) in graph.items()
    }


def crate_patterns(names, graph):
    seen, todo = set(), list(names)
    while todo:
        name = todo.pop()
        if name in seen or name not in graph:
            continue
        seen.add(name)
        todo += graph[name][1]
    return {graph[n][0] + "/**" for n in seen} | set(CARGO_FILES)


def inputs(
    job,
    repo=None,
    workflow_text=None,
    graph=None,
    crates=None,
    extra=None,
    jaic=None,
    via=None,
):
    """The input patterns of `job`, a job id of the workflow."""
    repo = repo or Repo()
    workflow_text = workflow_text if workflow_text is not None else repo.read(WORKFLOW)
    crates = CRATES if crates is None else crates
    extra = EXTRA if extra is None else extra
    block = jobs_of(workflow_text).get(job, "")
    text = step_text(repo, block)
    seeds = (
        (paths_in(repo, text, True) - {RUNNER})
        | set(extra.get(job, []))
        | runner_args(repo, text)
    )
    for action in actions_of(block):
        seeds.add(action + "/**")
    result = closure(repo, seeds, jaic, via)
    result |= seeds
    result.discard(RUNNER)
    names = crates.get(job, [])
    if names:
        graph = graph if graph is not None else crate_graph(repo)
        result |= crate_patterns(names, graph)
    return result


def collapse(patterns):
    """Drop patterns another `dir/**` in the set already covers."""
    dirs = [p[:-2] for p in patterns if p.endswith("/**")]
    return sorted(
        p for p in patterns if not any(p.startswith(d) and p != d + "**" for d in dirs)
    )


def main(argv):
    wanted = [a for a in argv if not a.startswith("-")]
    repo = Repo()
    text = repo.read(WORKFLOW)
    graph = crate_graph(repo)
    jaic = os.environ.get("JAIC_DEPS_JAIC")
    for problem in drift(text, only=GATED):
        print("drift:", problem, file=sys.stderr)
    for job in jobs_of(text):
        if wanted and job not in wanted:
            continue
        if not wanted and job not in GATED:
            continue
        print(f"== {job}")
        for pat in collapse(inputs(job, repo, text, graph, jaic=jaic)):
            print("  " + pat)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
