# Dependencies (`jacket dep`)

Jai projects share code as module folders under `modules/`, today mostly copied in by hand. `jacket dep` manages them as git submodules pinned in `jacket.toml`, with version ranges, version schemes and a release-age policy. The submodule's commit (its gitlink) is the lock, so there is no lockfile.

## Rules

- **Builds never touch the network.** `dep add`, `update` and `init` fetch, and only when run. `dep check` also asks each remote for its tags (to find moved ones) unless run with `--offline`. Builds use the pinned commits and an uninitialised submodule is an error, never a fetch.
- **Nothing is committed for you.** `dep` edits `jacket.toml` and the submodules; the user commits them.
- **No quiet acceptance.** A version that fails the policy, or whose age can't be checked, is an error with a note giving the override, even when the user named the version.
- **A dependency's `jacket.toml` is data.** `dep` reads its `[package]` and `[dependencies]` tables and nothing else. A dependency can never change how the program that uses it builds.
- **Git does the authentication.** `dep` runs the `git` on `PATH`, so SSH keys and credential helpers work as they do for any submodule. Git never prompts on the terminal. Before any change, `dep` checks that `git` is installed (else it prints the install command for the system's package manager) and that the project is a git repository (else it says to run `git init`). When a release-age policy applies and the command is not offline, `dep` also checks for `curl` first, with the same install note. A remote that cannot be read is reported with its address and a next step, as unreachable or as refused. Git runs with the repository-locating variables (`GIT_DIR` and the like) cleared, so `dep` works from a git hook, and with `ssh -o BatchMode=yes` unless `GIT_SSH_COMMAND`, `GIT_SSH` or `core.sshCommand` names one.
- **A failed step is undone.** Every command takes a snapshot of `jacket.toml` and the submodule commits first; when a step fails, including sharing and nested checkouts, added copies are removed, removed ones added back and moved ones moved back. `add` removes the submodule, its clone and its `.gitmodules` entry and restores `jacket.toml` when a step after the clone fails; `update` asks every remote and picks every version before changing anything, and if a later dependency fails it moves the ones already changed back to their old commits and `jacket.toml` to its old text; `remove` restores the entry and the checkout if the submodule cannot be removed; `added` is printed only after the dependencies are shared. When an undo itself fails, the error says what is left and the commands that clear it. A copy being replaced with `--name` is removed only after the remote and the release-age policy have been checked.
- **A relative local path is read from one place.** `jacket dep add ../lib` records the absolute path, so every command reaches the same repository.

## `jacket.toml`

```toml
[dependencies]
library  = { git = "github.com/someone/library", version = "^1.2" }   # path: modules/library
sdl_ext = { git = "https://codeberg.org/x/sdl_ext", rev = "3f9c2a1" }   # no tags: pinned by commit
utf8    = { git = "github.com/x/utf8", tag = "v0.4.1", min_release_age = "0" }   # policy waived for this one

[resolver]
min_release_age = "3d"
release_age_jitter = "2d"
seed = "k3v9q2xh"        # random, written by `dep init` or the first `dep add`; commit it
dedupe = false           # experimental; see Shared dependencies

[package]                # this project can be a dependency of others
name = "my_lib"          # the import name; default: the repo name
version = "1.2.0"        # optional; `dep add` warns when it differs from the tag
module = "src/My_Lib"    # the folder holding module.jai; default: the repo root
requires_jacket = ">=0.40"
platforms = ["macos", "linux", "windows"]
```

- `git`: `user/repo` means GitHub; a host path (`codeberg.org/x/y`) means HTTPS; a full URL or `git@...` is used as written.
- Exactly one of `version` (a range), `tag` or `rev` per entry. The submodule's commit is what builds; `jacket.toml` records what was asked for.
- `path`: defaults to `modules/<name>`. That folder is already searched, so builds need no new setting.
- `scheme` and `tag_prefix`: see Versions.

## Commands

| Command | Does |
| --- | --- |
| `jacket dep add <spec> [--name N] [--module P] [--scheme S] [--tag-prefix T]` | Adds a submodule and an entry. `<spec>` is `user/repo`, a host path or URL, with an optional `@<version-or-range>`, `@<tag>` or `@<rev>`. With no version, the newest stable one is pinned and recorded as `version = "^X.Y.Z"` (calendar and integer schemes record `tag`). A version written in full (`@1.2.3`) names that version's tag; a partial one (`@1.2`) or one with an operator is a range. |
| `jacket dep remove <name>` | Deinits the submodule and removes its folder and entry. |
| `jacket dep list [--all-tags] [--offline]` | Name, asked-for version, resolved tag and commit, path, and the newer version the range admits (asked of each remote for its tags; `--offline` skips it). `--all-tags` names the remote tags that are not versions. |
| `jacket dep update [name] [--to <tag>]` | Moves the submodule and the entry to the newest version the range and policy allow. |
| `jacket dep sync` | Checks out every pinned commit (like `git submodule update --init`, for the managed ones), for a fresh clone. |
| `jacket dep check [--offline]` | For CI: `jacket.toml` and the gitlinks agree, every tag still points at its pinned commit (here and on the remote), and every submodule is initialised. Asks the remote for its tags, never a forge API; `--offline` skips the remote. |
| `jacket dep init [--reseed]` | Adopts existing submodules under the module folders into `[dependencies]`, and writes `[resolver] seed`. |

A build that finds a managed submodule uninitialised fails with a note: `run jacket dep sync`. A module that sits below its repository's root (`module = "modules/Inner"`) is imported by its folder's name, so `jacket` adds that folder's parent to the program's `-import_dir`s.

### `dep init`

- Scans the module folders a compile would search (`<main_dir>/modules` and `[project] import_path`).
- **Submodules** become entries. The pin is the tag pointing at the checked-out commit (the highest semver one if several do), else the commit itself.
- **Vendored copies** (folders with no remote) are listed as "vendored, not managed" and left alone. `jacket dep add <spec> --name <folder>` replaces one with a pinned submodule from a source the user names.
- The policy applies: an adopted version that is too new, or whose age can't be checked, is listed with the override note.

## Versions

Tags come from the remote's ref listing (the `git ls-remote --tags` data), which works on every host with no API.

| Scheme | Tags | Notes |
| --- | --- | --- |
| `semver` (default) | `1.2.3`, `v1.2.3`, `1.2.3-rc.1` | Prereleases only when the range names one. Two-part tags (`0.8`, `v2.1`) read as `0.8.0`, `2.1.0`. |
| `calendar` | `2024.05.01`, `2024.5`, `24.05` | Compared field by field. |
| `integer` | `42`, `r42`, `v42` | One number. |

- `tag_prefix = "alpha-"` strips a fixed prefix first, for forms like `alpha-0.0.1` and `version-1.0`.
- Version ranges: `^1.2`, `~1.2.3`, `>=1.0, <2.0`, `=1.4.0`, `*`. Calendar and integer schemes take only the comparison forms.
- With no usable tags (about 80% of the corpus today), `add` pins the default branch's commit by `rev`, and `update` moves to its newest commit.
- Tags that don't parse under the scheme are skipped. `dep list --all-tags` shows them.

From a survey of the 190 corpus projects: 37 have tags; 27 use semver, 7 two-part, 6 a prefix, 7 other names; none use calendar versions; none use submodules, and 31 vendor copies of other modules.

## Release-age policy

- `min_release_age`: a version must have been published at least this long ago.
- When a range's newest version is too new and an older one is taken, `add` and `update` print a note naming the newer one and its age:

  ```text
  note: library v1.3.0 is newer but not used: it was published 20 hours ago; min_release_age is 3d
  ```
- `release_age_jitter`: adds a per-version offset in `[0, jitter]`, hashed from `seed`, package and version. Everyone on the project gets the same offset, and different projects get different ones, so a compromised release reaches projects in stages.
- With jitter set and no `seed`, `add` and `update` warn, apply no jitter, and suggest `jacket dep init`. The first `add` writes a seed if there is none.
- A commit (`rev`) has no release, so under an active policy it needs `min_release_age = "0"` on its entry. `dep init` adopts what it finds and prints the same error as a warning for each version that fails the policy.
- **Publish times come only from a forge's records:** the releases of GitHub (`github.com`), GitLab (`gitlab.com`) and Codeberg (`codeberg.org`). A tag with no release has no recorded time. Commit and tagger dates are never used, because whoever made the commit wrote them.
- **The policy applies to every version, named or not:**

```
$ jacket dep add someone/library@v1.3.0
error: library v1.3.0 was published 20 hours ago; min_release_age is 3d
note: to accept it, add `min_release_age = "0"` to its entry:
      library = { git = "github.com/someone/library", tag = "v1.3.0", min_release_age = "0" }
```

On a host without recorded times the error says the age can't be checked, with the same note.

### API use

- Publish times are cached permanently in jacket's user cache, keyed by host, repo, tag and commit. A moved tag misses the cache.
- Candidates are checked newest first, stopping at the first old-enough one.
- Release lists use conditional requests (`ETag`); GitHub's "not modified" answers don't count against its rate limit.
- On GitHub with `GITHUB_TOKEN` set, `update` asks for the release lists of every dependency that has no cached list in one GraphQL query. Cached lists are revalidated with their `ETag` instead, which costs no rate limit when nothing changed.
- A list is fetched at most once per run.
- `GITHUB_TOKEN` and `GITLAB_TOKEN`, when set, go only to that forge's API host. Codeberg and other Gitea or Forgejo hosts are read without a token.
- A rate-limited answer is an error that names the token to set.

## Moved tags

A tag can be pushed again to a different commit. `dep check` and `dep update` compare each pinned tag's commit on the remote with the gitlink and fail when they differ, naming the tag, both commits and the dependency.

## A dependency's `jacket.toml`

`dep` reads only `[package]` and `[dependencies]`:

- `[package] module` (or `--module`) says which folder holds `module.jai`. Without it, `dep add` uses the repository root, else the only `modules/<Name>/`, else fails and asks for `--module`. A module that isn't at the root is recorded as the entry's `module`.
- A module is imported by its folder's name. For a root module that is the submodule folder, so when `--name` is absent `dep add` names the folder after `[package] name`. For a module in a subfolder, `[package] name` must equal that folder's name or `add` fails.
- `dep add` refuses to replace an existing non-submodule folder unless `--name` is given.
- `version` is compared with the tag and a mismatch is a warning; `platforms` not listing this OS is a warning.
- `[package] import_path` (or the package's `[project] import_path` when it has no `[package]` key) applies **only to that package's files**. A file inside package P resolves `#import` in this order:
  1. `modules/` next to the file;
  2. P's own import paths;
  3. the program's roots (`<main_dir>/modules`, its import paths, the stdlib).

  So a dependency can't replace `Basic` or any other module for the rest of the program. `jacket build`, `check`, `run`, `test` and `lint` pass each dependency's folders as `-package_import_dir <dependency folder>=<folder>`; the language server doesn't apply them yet.
- `[dependencies]` gives the package's requirements as ranges. Without dedupe, `dep list` prints a `<dep> needs <name> <range>: pinned at <tag>` line for each and says when the nested pin doesn't satisfy it.
- `requires_jacket` is checked at `add` and `update`. A failing `add` removes the checkout; a failing `update` moves back to the old commit.
- Everything else (`[target]`, `[lint]`, `[fmt]`, `[experimental]`, runners) belongs to whoever builds the program and is never read from a dependency.

## Shared dependencies (`dedupe`)

Off by default and experimental.

- **Off:** each dependency's own dependencies are fetched recursively as its own submodules. A library's files find their copy through `modules/` next to them, which `search_roots` already tries first. Each copy is a separate module instance, so two copies have distinct types, and errors name each instance.
- **On** (`[resolver] dedupe = true`): after `add`, `update` and `remove`, the `[dependencies]` of every checked-out package are solved together with the project's entries.
  - A name no entry covers becomes an entry with `shared = true`, pinned by `tag` to the newest version every requirer admits and the release-age policy accepts, as a submodule of the project's `modules/`.
  - The nested submodules it replaces are never checked out (`dep sync` skips them too), so a library's `#import` falls through to the shared copy. The compiler doesn't change.
  - Entries the user wrote keep their pin and must satisfy every requirer. A shared entry stays while it fits everyone; plain `dep update` moves it to the newest version they admit.
  - Shared entries can't be removed or updated by name; the last package that needs one takes it with it. `dep list` marks them `(shared)` and `dep check` fails when a requirement is uncovered or unmet.
  - A conflict names each requirer and its range. A failing `dep add` removes the package and any copies only it needed. Requirers must agree on the repository, `scheme` and `tag_prefix`.
  - A package that keeps a plain copy of a module in the folder where its submodule would be can't share it; `add` warns.
- **Stress suite:** `crates/jacket/tests/dep_dedupe.rs` generates packages at several versions and compiles each shape with dedupe on and off. What it showed:
  - Diamond with a type in a public API: off, two copies of the type and a compile error (`argument of type Handle does not match parameter type Handle`); on, one copy and a clean build.
  - A file in a package's subfolder importing a nested module: off, `module not found` (no `modules/` next to it); on, it finds the shared copy.
  - Different `#module_parameters` per requirer: one copy of the files, two module instances, and the same type error on and off.
  - Chain, deep chain with a diamond, and a cycle of packages: one copy per name, settled in a few rounds.
  - Conflicting ranges, disagreeing schemes, a pinned entry the requirer rejects and a moved shared tag: each an error naming the packages.
  - A fresh clone: `dep sync` checks out the shared copy and leaves the nested ones empty.
- It stays off by default until it has run on real projects.

## How to change it

- **A command:** `crates/jacket/src/dep/commands.rs` (`dispatch` is the table); the clap definitions are `Action` in `dep/mod.rs`. Git and forge access go through the `Git` and `Forge` traits (`dep/git.rs`, `dep/forge.rs`), which tests replace with local bare repositories and a fake forge server (`crates/jacket/tests/dep_support/`).
- **`jacket.toml` fields:** `Dependency`, `Resolver` and `Package` in `crates/jacket-config/src/schema.rs`, then `editors/vscode/scripts/build.ts` and `node scripts/build.ts schemas`. Entries are written as text by `dep/manifest.rs`, which touches only `[dependencies]` lines and `[resolver] seed`, so comments and layout survive; add a new field to `manifest::inline` too.
- **Versions and ranges:** `dep/version.rs` (schemes, `parse_tag`, `parse_range`) and `dep/resolve.rs` (`classify`, `matching`).
- **The policy:** `dep/policy.rs` (ages, jitter) and `dep/forge.rs` (release lists, the cache, GraphQL batching).
- **A dependency's `[package]`:** `dep/package.rs` reads it, `dep/inspect.rs` applies it at `add` and reports requirements for `list`.
- **Module search:** `jacket_config::packages::dependency_roots` decides what each dependency adds; `jacket` turns it into `-import_dir` and `-package_import_dir` (`compile.rs`) and `jailint` into `ProgramInputs::packages`. The compiler side is `search_roots` in `crates/jaic/src/sema/modules/filesystem.rs`; see [module search order](../compiler/project.md#module-search-order). The language server doesn't read dependency import paths yet.
- **Sharing:** `dep/dedupe.rs`. The stress suite is `crates/jacket/tests/dep_dedupe.rs`; add a shape there with `observe`, which builds it with dedupe off and on and compiles both.
- **Gotchas:** `add` and `sync` check out only the dependency itself; `Git::nested` checks out what is inside it, skipping what the project shares. Git blocks local-path submodule URLs by default; `add` allows the one the user typed, and tests allow them through `GIT_CONFIG_*`.

## Configuration

- `[dependencies]`, `[resolver]` and `[package]` in `jacket.toml`, described above.
- `GITHUB_TOKEN`, `GITLAB_TOKEN`: sent only to their own forge's API host.
- `JACKET_FORGE_API`: replaces the forge API base URL, for tests.
- The release-time cache is `dep-release-times.json` under `jacket/` in the user cache folder (`XDG_CACHE_HOME`, `~/Library/Caches` or `LOCALAPPDATA`).

## Types

`types.rs` holds values parsed once where they enter `dep`: `CommitId` (a full hash from git), `Rev` (a user's abbreviated `rev`), `Target` (either, for a pin), `Revision` (a commit, abbreviation or tag to look up in a clone), `RefName` (a tag name), `SubPath` (a folder inside the project: relative, no `.` or `..` part) and `Age` (a release-age duration). `spec.rs` holds `Source` (a repository as written: path, URL or `owner/repo`), `RepoUrl` (an address git can fetch) and `At` (what follows `@` or `--to`). `version.rs` holds `Requirement` (a version range read under its scheme) and `Scheme`. `resolve.rs` holds `Asked`, what an entry of `jacket.toml` asks for: a range, a tag, a commit or nothing. The clap arguments parse into `Spec`, `At` and `Scheme` before a command runs. `Dependency` in `jacket-config` still holds strings read from `jacket.toml`; `Project::open` and `save` check each entry once (the repository, the folder and a single choice of `version`, `tag` or `rev`) and refuse one that fails, so a command that reads an entry into these types is not surprised. Functions take the types, so none checks its input again. The `[resolver]` policy is held as `Age` values in `Policy`.

## Dependencies

`git` on `PATH`, run only from `crates/jacket/src/git.rs` (`Repo` has one method per operation and owns the environment and the error text; `jacket migrate` uses it too; `dep/git.rs` holds the `Git` trait and the parsers of its output); `curl` on `PATH` for the forge APIs, run only from `forge.rs`; the `semver` crate; `jacket-config`. Related: [jacket](jacket.md), [module search order](../compiler/project.md#module-search-order), [toolchain discovery](../native/toolchain-discovery.md).
