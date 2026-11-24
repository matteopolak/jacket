# Plan: dependencies (`jacket dep`)

Status: D1 (direct dependencies by tag and commit), D2 (versions), D3 (release-age policy) and D4 (`[package]`) are built. The rest is plan; each milestone updates this page and the docs it names when it lands.

Jai projects share code as module folders under `modules/`, today mostly copied in by hand. `jacket dep` manages them as git submodules pinned in `jacket.toml`, with version ranges, version schemes and a release-age policy. The submodule's commit (its gitlink) is the lock, so there is no lockfile.

## Rules

- **Builds never touch the network.** `dep add`, `update` and `init` fetch, and only when run. `dep check` also asks each remote for its tags (to find moved ones) unless run with `--offline`. Builds use the pinned commits and an uninitialised submodule is an error, never a fetch.
- **Nothing is committed for you.** `dep` edits `jacket.toml` and the submodules; the user commits them.
- **No quiet acceptance.** A version that fails the policy, or whose age can't be checked, is an error with a note giving the override, even when the user named the version.
- **A dependency's `jacket.toml` is data.** `dep` reads its `[package]` and `[dependencies]` tables and nothing else. A dependency can never change how the program that uses it builds.
- **Git does the fetching and the authentication.** SSH keys and credential helpers work as they do for any submodule. `dep` needs the project to be a git repository; outside one, it errors with a `git init` note.

## `jacket.toml`

```toml
[dependencies]
jaison  = { git = "github.com/someone/jaison", version = "^1.2" }   # path: modules/jaison
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
| `jacket dep list [--all-tags] [--offline]` | Name, asked-for version, resolved tag and commit, path, and the newer version the range admits (asked of each remote with `git ls-remote`; `--offline` skips it). `--all-tags` names the remote tags that are not versions. |
| `jacket dep update [name] [--to <tag>]` | Moves the submodule and the entry to the newest version the range and policy allow. |
| `jacket dep sync` | Checks out every pinned commit (`git submodule update --init` for the managed ones), for a fresh clone. |
| `jacket dep check [--offline]` | For CI: `jacket.toml` and the gitlinks agree, every tag still points at its pinned commit (here and on the remote), and every submodule is initialised. Uses `git ls-remote`, never a forge API; `--offline` skips the remote. |
| `jacket dep init [--reseed]` | Adopts existing submodules under the module folders into `[dependencies]`, and writes `[resolver] seed`. |

A build that finds a managed submodule uninitialised fails with a note: `run jacket dep sync`.

### `dep init`

- Scans the module folders a compile would search (`<main_dir>/modules` and `[project] import_path`).
- **Submodules** become entries. The pin is the tag pointing at the checked-out commit (the highest semver one if several do), else the commit itself.
- **Vendored copies** (folders with no remote) are listed as "vendored, not managed" and left alone. `jacket dep add <spec> --name <folder>` replaces one with a pinned submodule from a source the user names.
- The policy applies: an adopted version that is too new, or whose age can't be checked, is listed with the override note.

## Versions

Tags come from `git ls-remote --tags`, which works on every host with no API.

| Scheme | Tags | Notes |
| --- | --- | --- |
| `semver` (default) | `1.2.3`, `v1.2.3`, `1.2.3-rc.1` | Prereleases only when the range names one. Two-part tags (`0.8`, `v2.1`) read as `0.8.0`, `2.1.0`. |
| `calendar` | `2024.05.01`, `2024.5`, `24.05` | Compared field by field. |
| `integer` | `42`, `r42`, `v42` | One number. |

- `tag_prefix = "alpha-"` strips a fixed prefix first, for forms like `alpha-0.0.1` and `version-1.0`.
- Ranges use Cargo's spelling: `^1.2`, `~1.2.3`, `>=1.0, <2.0`, `=1.4.0`, `*`. Calendar and integer schemes take only the comparison forms.
- With no usable tags (about 80% of the corpus today), `add` pins the default branch's commit by `rev`, and `update` moves to its newest commit.
- Tags that don't parse under the scheme are skipped. `dep list --all-tags` shows them.

From a survey of the 190 corpus projects: 37 have tags; 27 use semver, 7 two-part, 6 a prefix, 7 other names; none use calendar versions; none use submodules, and 31 vendor copies of other modules.

## Release-age policy

- `min_release_age`: a version must have been published at least this long ago.
- `release_age_jitter`: adds a per-version offset in `[0, jitter]`, hashed from `seed`, package and version. Everyone on the project gets the same offset, and different projects get different ones, so a compromised release reaches projects in stages.
- With jitter set and no `seed`, `add` and `update` warn, apply no jitter, and suggest `jacket dep init`. The first `add` writes a seed if there is none.
- A commit (`rev`) has no release, so under an active policy it needs `min_release_age = "0"` on its entry. `dep init` adopts what it finds and prints the same error as a warning for each version that fails the policy.
- **Publish times come only from a forge's records:** the releases of GitHub (`github.com`), GitLab (`gitlab.com`) and Codeberg (`codeberg.org`). A tag with no release has no recorded time. Commit and tagger dates are never used, because whoever made the commit wrote them.
- **The policy applies to every version, named or not:**

```
$ jacket dep add someone/jaison@v1.3.0
error: jaison v1.3.0 was published 20 hours ago; min_release_age is 3d
note: to accept it, add `min_release_age = "0"` to its entry:
      jaison = { git = "github.com/someone/jaison", tag = "v1.3.0", min_release_age = "0" }
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

  So a dependency can't replace `Basic` or any other module for the rest of the program.
- `[dependencies]` gives the package's requirements as ranges. Without dedupe, `dep list` prints a `<dep> needs <name> <range>: pinned at <tag>` line for each and says when the nested pin doesn't satisfy it.
- `requires_jacket` is checked at `add` and `update`. A failing `add` removes the checkout; a failing `update` moves back to the old commit.
- Everything else (`[target]`, `[lint]`, `[fmt]`, `[experimental]`, runners) belongs to whoever builds the program and is never read from a dependency.

## Shared dependencies (`dedupe`)

Off by default and experimental.

- **Off:** each dependency's own dependencies are fetched recursively as its own submodules. A library's files find their copy through `modules/` next to them, which `search_roots` already tries first. Each copy is a separate module instance, so two copies have distinct types, and errors name each instance.
- **On:** the ranges of every package are solved together, one version per name goes into the project's `modules/`, and the nested submodules it replaces are not checked out. A library's `#import` then falls through to the shared copy. The compiler doesn't change. A conflict is an error naming each requirer and its range. Vendored nested copies can't be shared and are reported.
- **Before it can become the default**, it needs its own stress suite:
  - generated packages at many versions, in diamond, chain and cycle shapes;
  - a dependency's type in another's public API;
  - different `#module_parameters` per requirer;
  - mixed schemes, moved tags and deep nesting;
  - imports from a library's subfolders, which skip its own `modules/`;
  - every combination compiled with dedupe on and off, and the results compared.

## Milestones

1. **D1, direct dependencies.** `add`, `remove`, `list`, `update`, `sync`, `check` and `init`; `[dependencies]` with `tag` and `rev`; moved-tag checks.
2. **D2, versions.** Schemes, prefixes and ranges; `version =` entries.
3. **D3, release-age policy.** `[resolver]`, forge time lookups, caching, `seed`.
4. **D4, `[package]`.** Name, module path, `requires_jacket`, `platforms`; `jacket new --lib` writes the table.
5. **D5, package-scoped import paths.** The compiler change (per-package search roots, passed by jacket), with fixtures in `crates/jaic-cli/tests/module_r*`.
6. **D6, dedupe.** The solver and the stress suite, behind `dedupe = true`.

## Files

- CLI: `crates/jacket/src/cli.rs`, with the commands in a new `dep` module of the jacket crate.
- Settings: `crates/jacket-config/src/schema.rs` and the editor schema `editors/vscode/schemas/jacket.schema.json`.
- Module search: `crates/jaic/src/sema/modules/filesystem.rs` (`search_roots`) and `docs/compiler/project.md`.
- Related: [jacket](jacket.md), [module search order](../compiler/project.md#module-search-order), [toolchain discovery](../native/toolchain-discovery.md).
