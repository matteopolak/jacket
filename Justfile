# Developer commands. `just --list` shows them; docs/tools/development.md explains them.
# The logic lives in tools/dev/dev.py and tools/dev/*.sh; CI runs these recipes too, so a local run and
# CI cannot drift apart. Needs just >= 1.58 (recipe options).

set shell := ["sh", "-cu"]
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

py := if os() == "windows" { "python" } else { "python3" }

# The Python scripts import their cross-folder helpers as `from toolkit.repo import ROOT`; every recipe sees tools/ on the module path.
export PYTHONPATH := justfile_directory() / "tools"

# Parallel cargo jobs. 3 suits a 16 GiB machine; JAI_JOBS or --jobs overrides it.
default_jobs := env("JAI_JOBS", "3")

[private]
default:
    @{{ just_executable() }} --list --unsorted

# Build jaic, jailsp, jailint and target/jaifmt, plus the native libraries and helpers a release ships (--native=false skips them).
[group('build')]
[arg("release", long, value="true")]
[arg("native", long)]
[arg("jobs", long, short="j")]
build release="false" native="true" jobs=default_jobs:
    {{ py }} tools/dev/dev.py build -j {{ jobs }} {{ if release == "true" { "--release" } else { "" } }} {{ if native == "false" { "--no-native" } else { "" } }}

# Format Jai (jaifmt) and Rust (rustfmt, item spacing); paths narrow it to files or directories.
[group('check')]
[arg("check", long, value="true")]
[arg("staged", long, value="true")]
[arg("lang", long, pattern="jai|rust|all")]
[arg("jobs", long, short="j")]
fmt check="false" staged="false" lang="all" jobs=default_jobs *paths:
    {{ py }} tools/dev/dev.py fmt -j {{ jobs }} --lang {{ lang }} {{ if check == "true" { "--check" } else { "" } }} {{ if staged == "true" { "--staged" } else { "" } }} {{ paths }}

# Lint Jai (jailint -D warnings) and Rust (clippy -D warnings); --fix applies the safe fixes.
[group('check')]
[arg("fix", long, value="true")]
[arg("staged", long, value="true")]
[arg("release", long, value="true")]
[arg("lang", long, pattern="jai|rust|all")]
[arg("os", long, pattern="|linux|windows|macos|wasm")]
[arg("jobs", long, short="j")]
lint fix="false" staged="false" release="false" lang="all" os="" jobs=default_jobs *paths:
    {{ py }} tools/dev/dev.py lint -j {{ jobs }} --lang {{ lang }} {{ if os != "" { "--os " + os } else { "" } }} {{ if release == "true" { "--release" } else { "" } }} {{ if fix == "true" { "--fix" } else { "" } }} {{ if staged == "true" { "--staged" } else { "" } }} {{ paths }}

# Check that the VS Code extension's generated grammar and schemas are current (skipped without node 24).
[group('check')]
editor:
    {{ py }} tools/dev/dev.py editor

# Run the Neovim plugin's headless tests (editors/nvim); builds a debug jailsp for the end-to-end ones. Skipped without nvim 0.11.
[group('test')]
[arg("unit", long, value="true")]
[arg("jobs", long, short="j")]
nvim-test unit="false" jobs=default_jobs:
    {{ py }} tools/dev/dev.py nvim -j {{ jobs }} {{ if unit == "true" { "--unit" } else { "" } }}

# What CI's format and lint steps run: fmt --check, then lint, over everything, plus the editor checks.
[group('check')]
[arg("jobs", long, short="j")]
check jobs=default_jobs:
    {{ just_executable() }} fmt --check --jobs {{ jobs }}
    {{ just_executable() }} lint --jobs {{ jobs }}
    {{ just_executable() }} editor
    {{ just_executable() }} nvim-test --unit

# Run the cargo tests (a crate and a test-name filter narrow them) and the tools/ unittests.
[group('test')]
[arg("suite", long, pattern="cargo|tools|all")]
[arg("crate", long)]
[arg("jobs", long, short="j")]
test suite="all" crate="" jobs=default_jobs *filter:
    {{ py }} tools/dev/dev.py test -j {{ jobs }} --suite {{ suite }} {{ if crate != "" { "-p " + crate } else { "" } }} {{ filter }}

# Turn on the hooks in .githooks for this clone and its worktrees (a relative core.hooksPath).
[group('setup')]
hooks:
    {{ py }} tools/dev/dev.py hooks

# The text checks CI runs on every push: style (file length, banners), paths (repo paths named in docs), ci-sources (include_str! inputs). Names narrow them.
[group('check')]
repo-checks *names:
    {{ py }} tools/dev/dev.py repo-checks {{ names }}

# Set the compiler version in Cargo.toml, both lock files, the Neovim plugin and the CHANGELOG header (tools/bump_version.py).
[group('setup')]
bump-version version:
    {{ py }} tools/bump_version.py {{ version }}

# Download the pinned upstream Jai projects into corpus/upstream (inert test input).
[group('setup')]
fetch-upstreams *args:
    {{ py }} tools/fetch_upstreams.py {{ args }}

# Build the browser compiler bundle (wasm32) into target/wasm.
[group('web')]
wasm *args:
    {{ py }} tools/browser/build_scripting_wasm.py --release --output target/wasm {{ args }}

# Open VS Code with only the freshly built extension, on FOLDER or a temporary copy of examples/.
[group('web')]
[arg("debug", long, value="true")]
[arg("jobs", long, short="j")]
vscode-try debug="false" jobs=default_jobs folder="":
    {{ py }} tools/dev/dev.py vscode-try -j {{ jobs }} {{ if debug == "true" { "--debug" } else { "" } }} {{ folder }}

# Run an editors/vscode package script (lint, test, build, package, ...) through pnpm.
[group('web')]
[working-directory('editors/vscode')]
vscode script="lint":
    pnpm install --frozen-lockfile --ignore-scripts
    pnpm run {{ script }}
