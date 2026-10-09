# Developer commands. `just --list` shows them; docs/tools/justfile.md explains them.
# The logic lives in tools/dev.py and tools/*.sh; CI runs these recipes too, so a local run and
# CI cannot drift apart. Needs just >= 1.58 (recipe options).

set shell := ["sh", "-cu"]
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

py := if os() == "windows" { "python" } else { "python3" }

# Parallel cargo jobs. 3 suits a 16 GiB machine; JAI_JOBS or --jobs overrides it.
default_jobs := env("JAI_JOBS", "3")

[private]
default:
    @{{ just_executable() }} --list --unsorted

# Build jaic, jailsp and jailint with cargo, then target/jaifmt with jaic.
[group('build')]
[arg("release", long, value="true")]
[arg("jobs", long, short="j")]
build release="false" jobs=default_jobs:
    {{ py }} tools/dev.py build -j {{ jobs }} {{ if release == "true" { "--release" } else { "" } }}

# Format Jai (jaifmt) and Rust (rustfmt, item spacing); paths narrow it to files or directories.
[group('check')]
[arg("check", long, value="true")]
[arg("staged", long, value="true")]
[arg("lang", long, pattern="jai|rust|all")]
[arg("jobs", long, short="j")]
fmt check="false" staged="false" lang="all" jobs=default_jobs *paths:
    {{ py }} tools/dev.py fmt -j {{ jobs }} --lang {{ lang }} {{ if check == "true" { "--check" } else { "" } }} {{ if staged == "true" { "--staged" } else { "" } }} {{ paths }}

# Lint Jai (jailint -D warnings) and Rust (clippy -D warnings); --fix applies the safe fixes.
[group('check')]
[arg("fix", long, value="true")]
[arg("staged", long, value="true")]
[arg("lang", long, pattern="jai|rust|all")]
[arg("jobs", long, short="j")]
lint fix="false" staged="false" lang="all" jobs=default_jobs *paths:
    {{ py }} tools/dev.py lint -j {{ jobs }} --lang {{ lang }} {{ if fix == "true" { "--fix" } else { "" } }} {{ if staged == "true" { "--staged" } else { "" } }} {{ paths }}

# What CI's format and lint steps run: fmt --check, then lint, over everything.
[group('check')]
[arg("jobs", long, short="j")]
check jobs=default_jobs:
    {{ just_executable() }} fmt --check --jobs {{ jobs }}
    {{ just_executable() }} lint --jobs {{ jobs }}

# Run the cargo tests (a crate and a test-name filter narrow them) and the tools/ unittests.
[group('test')]
[arg("suite", long, pattern="cargo|tools|all")]
[arg("crate", long)]
[arg("jobs", long, short="j")]
test suite="all" crate="" jobs=default_jobs *filter:
    {{ py }} tools/dev.py test -j {{ jobs }} --suite {{ suite }} {{ if crate != "" { "-p " + crate } else { "" } }} {{ filter }}

# Turn on the pre-commit hook (.githooks/pre-commit) for this clone.
[group('setup')]
hooks:
    git config core.hooksPath .githooks

# Download the pinned upstream Jai projects into corpus/upstream (inert test input).
[group('setup')]
fetch-upstreams *args:
    {{ py }} tools/fetch_upstreams.py {{ args }}

# Build the browser compiler bundle (wasm32) into artifacts/wasm.
[group('web')]
wasm *args:
    {{ py }} tools/build_scripting_wasm.py --release --output artifacts/wasm {{ args }}

# Run an editors/vscode package script (lint, test, build, package, ...) through pnpm.
[group('web')]
[working-directory('editors/vscode')]
vscode script="lint":
    pnpm install --frozen-lockfile --ignore-scripts
    pnpm run {{ script }}
