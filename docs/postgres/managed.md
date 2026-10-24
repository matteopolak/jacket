# Managed PostgreSQL

## What it is

`stdlib/Extensions/Postgres/managed.jai` starts a private, throwaway PostgreSQL server with no setup: it downloads pinned binaries once, verifies them, caches them per user, creates a cluster and starts it. The compile-time describe service uses it when a project has no `DATABASE_URL` and managed mode is on (the hookup is a separate change; see the design's plan item 6).

```jai
settings: Pg.Managed_Settings;
server, error := Pg.start_managed(settings);
if error { print("%\n", error); return; }
defer Pg.stop_managed(*server);

Io.io_run(() { db, pg_error := Pg.connect(server.config); ... });
```

## How it works

1. `managed_triple()` picks the release asset name from `OS` and `CPU` (Linux uses the musl build when `/lib/ld-musl-*.so.1` exists, otherwise glibc).
2. `ensure_binaries` looks for `<cache>/<version>-<triple>/bin/pg_ctl`. If missing it downloads `postgresql-<version>-<triple>.tar.gz` from the `theseus-rs/postgresql-binaries` release with `curl`, hashes it in process and compares with the SHA-256 committed in `MANAGED_ARCHIVES` (never a checksum fetched with it), unpacks with `tar` into a private temporary folder under the cache, and renames that folder into place. Two processes racing both unpack; the first rename wins and the other uses it. The temporary folder is always removed.
3. `start_managed` creates the cluster with `initdb -U postgres --auth=trust -E UTF8 --locale=C` and leaves a `.jacket-managed` marker. `postgresql.conf` gets one `include_if_exists = 'jacket.conf'` line; `jacket.conf` is rewritten on every start with the port, the listen setting, `fsync = off`, `synchronous_commit = off`, `full_page_writes = off`, `max_connections = 40`, `shared_buffers = 32MB` and `Managed_Settings.extra_conf`.
4. It runs `pg_ctl status`; if a server already runs on the data folder, it is reused: port and socket folder are read from `postmaster.pid`. Otherwise `pg_ctl start -w -l <data>/server.log`.
5. The server listens on a Unix socket only (`listen_addresses = ''`), in `<data>/sock`, or a short `/tmp/jacket-pg-<hash>` folder when that path would not fit in a socket address (about 100 bytes). Windows listens on `127.0.0.1` at a port from the dynamic range. The returned `Managed_Server.config` connects as `postgres` to database `postgres` with no TLS; `url` is the same as a URL.

`stop_managed` runs `pg_ctl stop -m fast`; it only stops a server this process started unless `force` is set. The data folder stays, so the next start is quick (about 0.1 s against about 5 s for the first start, which includes the download).

Defaults: cache `$XDG_CACHE_HOME/jacket/postgres` or `~/.cache/jacket/postgres` (Linux), `~/Library/Caches/jacket/postgres` (macOS), `%LOCALAPPDATA%\jacket\postgres` (Windows); data `<cache>/cluster-<major>`.

## How to change it

- **Pinned releases.** `MANAGED_ARCHIVES` holds the latest release of each supported major: 15.19.1, 16.15.1, 17.11.1 (the default, `MANAGED_VERSION`) and 18.6.1; select one with `Managed_Settings.version`. 15.19.1 has no `aarch64-pc-windows-msvc` asset upstream, so that row is absent. Each digest was taken from the `.sha256` file and matched against the GitHub API digest. Each major gets its own default cluster folder.
- **Bump the pinned version.** In the release listing, take the new tag (the asset names are `postgresql-<version>-<triple>.tar.gz`, tag equals version). For each triple get the digest from `<asset>.sha256` and confirm it against the digest the GitHub API reports for the asset (`gh api repos/theseus-rs/postgresql-binaries/releases/tags/<tag> --jq '.assets[] | [.name,.digest]'`). Replace the rows in `MANAGED_ARCHIVES` and `MANAGED_VERSION`. Keep old rows only if they should stay selectable through `Managed_Settings.version`. Run `tests/managed.jai` with `JAI_PG_TEST_MANAGED=1`, and update the major-version check in `check_server` for a new major. A new major needs a new cluster: the default data folder name includes it.
- **Add a platform.** Add it to `managed_triple_for` and a row per release in `MANAGED_ARCHIVES`.
- **Change the server settings.** Edit `managed_conf` (or pass `extra_conf`). Existing clusters pick it up on the next start because `jacket.conf` is rewritten.
- Gotchas: a data folder with `PG_VERSION` but no `.jacket-managed` marker is refused (it is either someone's cluster or an interrupted `initdb`; delete it). Starting the same data folder from two processes at once is not safe; the describe service is meant to be its single owner. The Windows path is written to the same contract but has not been run.

## Configuration

`Managed_Settings`: `version`, `cache_dir`, `data_dir`, `socket_dir`, `port`, `timeout_seconds`, `extra_conf`. Environment read for the default cache folder: `XDG_CACHE_HOME`, `HOME` (`USERPROFILE` and `LOCALAPPDATA` on Windows). Constants: `MANAGED_VERSION`, `MANAGED_ARCHIVES`, `MANAGED_RELEASE_URL`.

Test: `stdlib/Extensions/Postgres/tests/managed.jai` runs the pure checks by default. `JAI_PG_TEST_MANAGED=1` downloads into a temporary cache, starts a server on port 54339, runs `select version()`, checks reuse and restart, and removes everything.

## Dependencies

- `curl` and `tar` on the `PATH` (both ship with macOS, most Linux distributions and Windows 10 and later). Nothing else is needed; an error names the missing tool.
- Network access to `github.com` on the first run only.
- `Process` (to run them), `File`/`File_Utilities`, and the module's own SHA-256 and connection code.
- The prebuilt server binaries from [theseus-rs/postgresql-binaries](https://github.com/theseus-rs/postgresql-binaries) (PostgreSQL's licence; each archive carries its notices).
