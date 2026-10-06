# Dependency policy

## What it is

Cargo may only select registry crate versions that were published at least 14 days ago, for direct and transitive dependencies alike. A dependency taken from a git repository instead must be pinned to a full commit hash that landed upstream at least 14 days ago. `jaic` itself has no external dependencies; the external crates are `serde`/`serde_json` (language server) and `inkwell`/`llvm-sys` (LLVM backend). `inkwell` currently comes from git (see below).

## How it works

`.cargo/config.toml` enables the nightly `min-publish-age` feature (`registry.global-min-publish-age = "14 days"`, resolver `incompatible-publish-age = "deny"`); `rust-toolchain.toml` pins a nightly that supports it. Cargo permits existing lockfile entries even if they are young, so `python3 tools/check_dependency_age.py` independently checks every locked crate against the crates.io API and fails closed on unknown sources, missing metadata, yanked releases and versions younger than 14 days. Internal path crates are skipped.

Cargo's `min-publish-age` does not cover git sources, so the checker does. It accepts a git source only in the form Cargo locks a `rev = "<40 hex digits>"` dependency on GitHub (`git+https://github.com/<owner>/<repo>?rev=<sha>#<sha>`, requested and resolved hash equal) and asks the GitHub API for that commit's committer date, which must be at least 14 days old. Branches, tags, short revisions and other hosts still fail as unverified.

CI runs the checker before any build (`ci.yml`, `release.yml`, `browser-release.yml`, and `fuzz.yml` for `fuzz/Cargo.lock`). The steps that check `Cargo.lock` pass the workflow's read-only `GITHUB_TOKEN`, so the GitHub API's rate limit is not shared with everything else on the runner's address.

### The inkwell pin

Inkwell 0.10.0, the newest release, supports LLVM up to 22. LLVM 23 support (`llvm23-1`, llvm-sys 231) is on its main branch: TheDan64/inkwell#702, plus #711, which masks `const_int` values to the type's width because LLVM 23 no longer truncates them implicitly. The root `Cargo.toml` pins commit `b7cbeed24af8`, the newest main commit that was 14 days old when jaic moved to LLVM 23. Both `inkwell` and `inkwell_internals` resolve to it; `llvm-sys` still comes from crates.io.

When a crates.io release with `llvm23-1` is 14 days old, switch back to `version = "..."`, and drop the comment in `Cargo.toml` and `allowBuiltinFetchGit` in `nix/jaic.nix`.

## How to change it

Dependencies used by several crates go in `[workspace.dependencies]` of the root `Cargo.toml`. Prefer exact versions (`=x.y.z`, as the serde crates are); a registry `inkwell` uses a plain requirement and relies on the lockfile. A git dependency needs `rev` with the full hash (not `branch` or `tag`), and only when no aged release provides what is needed. Update with the pinned nightly Cargo, then run the checker before building. Never weaken the age policy to resolve a conflict. Tests: `tools/test_dependency_age.py`.

## Configuration

`--lockfile PATH` (default `Cargo.lock`). `GITHUB_TOKEN`, when set, authenticates the GitHub requests. Policy settings live in `.cargo/config.toml`.

## Dependencies

Python 3.11+ standard library, and network access to crates.io's version API and, for git sources, GitHub's commits API (missing metadata is a failure).
