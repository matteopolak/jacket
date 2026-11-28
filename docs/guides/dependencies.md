# Dependencies

`jacket dep` adds other people's Jai modules to your project as git submodules. Each dependency is pinned to one commit, and `jacket.toml` records the version range you asked for. There is no lockfile: the submodule's commit is the lock.

> [!WARNING]
> `jacket dep` is new and may change in any release.

## Add a dependency

Your project has to be a git repository. Then:

```sh
jacket dep add someone/library
```

This clones `github.com/someone/library` into `modules/library`, checks out its newest stable release, and adds an entry to `jacket.toml`:

```toml
[dependencies]
library = { git = "github.com/someone/library", version = "^1.2.0" }
```

`modules/` next to your program is already searched by `#import`, so the module works with no other setting:

```jai
#import "library";
```

Commit `jacket.toml`, `.gitmodules` and `modules/library` together. `jacket dep` never commits for you.

### Where a dependency comes from

| You write | It means |
| --- | --- |
| `someone/library` | GitHub |
| `codeberg.org/someone/library` | that host, over HTTPS |
| `https://...` or `git@...` | that URL, as written |

SSH keys and git credential helpers work as they do for any clone.

### Choosing a version

Add `@` and a version, tag or commit:

```sh
jacket dep add someone/library@^1.2      # a range: the newest 1.x from 1.2 up
jacket dep add someone/library@1.3.0     # exactly the 1.3.0 release
jacket dep add someone/library@3f9c2a1   # a commit, for projects with no releases
```

Version ranges: `^1.2`, `~1.2.3`, `>=1.0, <2.0`, `=1.4.0`, `*`. Tags such as `v1.2.3`, `1.2.3` and two-part `0.8` are understood. For other tag forms, pass `--scheme calendar` (`2024.05.01`), `--scheme integer` (`r42`), or `--tag-prefix alpha-` (`alpha-0.0.1`).

A repository with no version tags is pinned by commit, and `jacket dep update` moves it to the newest commit of its default branch.

## Day to day

```sh
jacket dep list               # each dependency, its pin, and newer versions its range allows
jacket dep update             # move every dependency to the newest version its range allows
jacket dep update library      # just one
jacket dep remove library
```

After cloning a project that has dependencies, check them out with:

```sh
jacket dep sync
```

A build never downloads anything. If a dependency isn't checked out, the build fails and tells you to run `jacket dep sync`.

### In CI

```sh
jacket dep check
```

It fails when `jacket.toml` and the pinned commits disagree, when a submodule isn't checked out, or when a release tag on the remote now points at a different commit than the one you pinned.

### Existing submodules

If your project already has modules as git submodules, `jacket dep init` adds entries for them. Folders copied in by hand are listed and left alone; replace one with `jacket dep add <source> --name <folder>`.

## Waiting before taking a new release

A release that was published minutes ago has had no time for anyone to notice a problem with it. You can require new versions to be a few days old:

```toml
[resolver]
min-release-age = "3d"
release-age-jitter = "2d"
seed = "k3v9q2xh"
```

- `min-release-age`: `add` and `update` skip releases younger than this (default two weeks; `0` turns the policy off). When they skip one for an older version, a note names it and says why.
- `release-age-jitter`: adds up to this much extra wait (default three days), different for each version. Everyone on the project waits the same amount, because it comes from `seed`.
- `seed`: random text that `jacket dep init` or the first `jacket dep add` writes. Commit it.

The check applies to versions you name yourself too:

```
$ jacket dep add someone/library@v1.3.0
error: library v1.3.0 was published 20 hours ago; min-release-age is 3d
note: to accept it, add `min-release-age = "0"` to its entry:
      library = { git = "github.com/someone/library", tag = "v1.3.0", min-release-age = "0" }
```

Publish times come from the releases on GitHub, GitLab and Codeberg, never from commit dates, which whoever made the commit can set. A dependency hosted elsewhere, or a commit pin, has no publish time, so it needs `min-release-age = "0"` on its entry while the policy is on. Set `GITHUB_TOKEN` or `GITLAB_TOKEN` if you hit a rate limit; each token is sent only to its own host.

## Publishing a library

Add a `[package]` table to your library's `jacket.toml`:

```toml
[package]
name = "my_lib"                # what users #import; default: the repository's name
version = "1.2.0"
module = "src/My_Lib"          # the folder with module.jai; default: the repository root
jacket-version = ">=0.40"
platforms = ["macos", "linux", "windows"]
import-path = ["vendor"]       # extra folders your own files import from

[dependencies]
utf8 = { git = "github.com/x/utf8", version = "^0.4" }
```

- Tag releases with their version (`v1.2.0`) and publish them as releases on your forge, so users with a release-age policy can take them.
- `import-path` applies only to your library's files. Your library can't change which `Basic` or other module the rest of a program gets.
- Only `[package]` and `[dependencies]` are read from a dependency. Its targets, lint and format settings never affect the program that uses it.

Your library's own dependencies are shared: `jacket dep` keeps one copy of each module per group of compatible versions (the same `^1` or `0.minor` rule as Cargo), so two libraries that share a type through a third see the same type. Set `dedupe = false` under `[resolver]` to give each library its own nested copy instead.

## Where to go next

- [Dependencies reference](../tools/dependencies.md): every field and command, version schemes, the publish-time cache and shared dependencies.
- [Tools](../reference/tools.md#dependencies): the `jacket dep` flags.
- [Module search order](../compiler/project.md#module-search-order): which folders `#import` looks in.
