# Managed PostgreSQL

`stdlib/Extensions/Postgres/managed.jai` starts a private, throwaway PostgreSQL server with no setup: it downloads pinned binaries once, verifies them, caches them per user, creates a cluster and starts it. The compile-time describe service uses it when a project has no database URL and managed mode is on.

```jai
settings: Pg.Managed_Settings;
server, error := Pg.start_managed(settings);
if error { print("%\n", error); return; }
defer Pg.stop_managed(*server);

Io.io_run(() { db, pg_error := Pg.connect(server.config); ... });
```

## How it works

1. `managed_triple()` picks the release asset from `OS` and `CPU` (musl on Linux when `/lib/ld-musl-*.so.1` exists).
2. `ensure_binaries` looks in `<cache>/<version>-<triple>/bin/pg_ctl`. If missing it downloads the archive from `theseus-rs/postgresql-binaries` with `curl`, checks it against the SHA-256 committed in `MANAGED_ARCHIVES` (never one fetched with it), unpacks with `tar` into a temporary folder and renames that into place. Racing processes both unpack; the first rename wins.
3. `start_managed` runs `initdb` (trust auth, UTF-8, C locale) and leaves a `.jacket-managed` marker. `jacket.conf` is rewritten on every start with the port and fast, unsafe settings (`fsync = off` and friends) plus `Managed_Settings.extra_conf`.
4. If `pg_ctl status` says a server already runs on the data folder, it is reused. Otherwise `pg_ctl start -w`.
5. The server listens on a Unix socket only (a short `/tmp/jacket-pg-<hash>` folder if the path would be too long), or on `127.0.0.1` on Windows. `Managed_Server.config` connects as `postgres`, no TLS.

`stop_managed` runs `pg_ctl stop -m fast` and only stops a server this process started, unless `force`. The data folder stays, so the next start takes about 0.1 s instead of about 5 s.

Cache: `$XDG_CACHE_HOME/jacket/postgres` or `~/.cache/jacket/postgres` (Linux), `~/Library/Caches/jacket/postgres` (macOS), `%LOCALAPPDATA%\jacket\postgres` (Windows). Data: `<cache>/cluster-<major>`.

## Compile-time use

[Typed queries](typed-queries.md) pick the managed server when there is no URL, `JACKET_PG_OFFLINE` is unset and managed mode is on. The pg tool's `prepare`, `check`, `generate` and `diff` follow the same rule. A good cache entry still wins; only a miss starts the server.

- **Migrations are required.** A managed server starts empty, so its schema comes from `db/migrations`. With none, a miss is a compile error.
- **Two steps.** The first start downloads about 30 MB, longer than a service request may take. The build calls `prepare_managed` (download, `initdb`) once; the describe service only calls `start_managed`.
- **The service owns the server.** It starts it on first need and does not stop it on exit, which is why the next build is quick. Stop it with `pg_ctl stop -D <cache>/cluster-<major>`.

## How to change it

- **Bump a release.** Take the new tag, get each asset's digest from its `.sha256` and check it against `gh api repos/theseus-rs/postgresql-binaries/releases/tags/<tag> --jq '.assets[] | [.name,.digest]'`. Replace the rows in `MANAGED_ARCHIVES` and `MANAGED_VERSION`, update the major check in `check_server` for a new major, and run `tests/managed.jai` with `JAI_PG_TEST_MANAGED=1`. Each major gets its own cluster folder.
- **Add a platform.** `managed_triple_for` plus a row per release in `MANAGED_ARCHIVES`.
- **Server settings.** `managed_conf`, or pass `extra_conf`.
- A data folder with `PG_VERSION` but no marker is refused (someone's cluster, or an interrupted `initdb`); delete it. Starts take a lock folder `<data_dir>.lock`; on Windows a stuck lock gives up after 10 minutes and names the folder. Windows unpacks with System32's `tar.exe`.

## Configuration

`Managed_Settings`: `version`, `cache_dir`, `data_dir`, `socket_dir`, `port`, `timeout_seconds`, `extra_conf`. Pinned: 15.19.1, 16.15.1, 17.11.1 (default), 18.6.1.

```toml
[postgres]
managed = true
managed_version = "16.15.1"
```

Environment (wins over `jacket.toml`): `JACKET_PG_MANAGED` (`0` turns it off), `JACKET_PG_MANAGED_VERSION`, `JACKET_PG_MANAGED_CACHE`.

## Dependencies

`curl` and `tar` on `PATH`, network access to `github.com` on the first run, `Process` and `File_Utilities`. The binaries are from [theseus-rs/postgresql-binaries](https://github.com/theseus-rs/postgresql-binaries). Tests: `tests/managed.jai` (pure checks by default; `JAI_PG_TEST_MANAGED=1` downloads and starts a real one).
