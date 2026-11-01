# Dogfood applications

Real applications written in Jai under `examples/`, to exercise the compiler, stdlib and extensions together. Each keeps a `FRICTION.md` of rough edges found while writing it. They are not in CI, except that `jaic -check` runs over their programs (`crates/jaic-cli/tests/examples.rs`).

- `examples/game2d/`: a 2D shooter plus a JSON backend. `shared/` (API types, typed client), `server/` (Http, Postgres, Tls, Json; `--listen`, `--database`, `--threads`, `--pool-size`, `--tls-cert`/`--tls-key`), `loadtest/` (Io fibers; reports throughput and p50/p99/max), `game/` (Simp, window, input, sound, zip asset pack, hot reload, `--smoke` and `--autoplay N`), and `examples/game2d/tools/assetgen/`.
- `examples/game3d/`: one program (`game/main.jai`): `Soa` entities updated with `Extensions/SIMD`, a job system on `Thread` and `Atomics`, meshoptimizer meshes, WebGPU rendering, an MPEG-1 intro via `pl_mpeg`. See its README.
- `examples/kvdb/`: a key-value database. `store/` (checksummed append-only log, recovery, compaction; md5, File), `protocol/` (line protocol, server and client on `Io` fibers), `main.jai` (`Extensions/Args` subcommands), and `tests/`, which `jacket test` runs in that folder. See its README.
- `examples/sitegen/`: a static site generator. `markdown/` (a Markdown subset to HTML), `site/` (template, pages in a `Xar`, assets named by an `md5` digest, small images inlined with `Base64`, one `Flat_Pool` per build), `main.jai` (`Extensions/Args`), a `demo/` site, and `tests/`, which `jacket test` runs in that folder. See its README.
- `examples/chat/`: a multi-room chat server, terminal client and command bot on `Io` fibers, with TLS, history recovery, a rate limiter, SMTP through `Mail` on an OS worker, and stats over `Shared_Memory_Channel`. `tests/` runs under `jacket test`. See its README.
- `examples/chip8/`: a CHIP-8 and SUPER-CHIP interpreter. The CPU is a deterministic library (no files, clock or globals) checked by framebuffer hashes from `Extensions/Meow_Portable`; the window uses Simp and `Sound_Player`. See its README.
- `examples/imageview/`: a folder image browser on `Extensions/UI`: thumbnail pages decoded on `Thread_Group` workers into a byte-capped LRU cache, zoom, pan, rotate, animation and export. See its README.
- `examples/editor/`: a native text editor on `Extensions/UI` and `Extensions/Desktop` with a `jailsp` client. See [the editor doc](../extensions/ui-editor.md) and its README.
- `examples/seek/`: a small grep. `search/` (file walk with `visit_files`, globs, `Extensions/Regex` or fixed-text matching, one `Thread_Group` job per file with a buffer per worker, output joined in path order), `main.jai` (`Extensions/Args`), and `tests/`, which `jacket test` runs in that folder. See its README.

## Running

```sh
just dogfood                                # server, load test (HTTP and HTTPS), game smoke
sh examples/game2d/tools/smoke_server.sh    # backend only
sh examples/game2d/tools/smoke_game.sh      # game only; FRAME=out.png keeps the offscreen frame
sh examples/game3d/tools/smoke_game.sh      # 3D game
sh examples/kvdb/tools/smoke.sh             # key-value database: CLI, recovery, server
(cd examples/kvdb && jacket test)           # its unit tests
sh examples/sitegen/tools/smoke.sh          # static site generator: builds the demo site and checks it
(cd examples/sitegen && jacket test)        # its unit tests
sh examples/seek/tools/smoke.sh             # grep: every flag, -j 1 against -j 8, exit codes
(cd examples/seek && jacket test)           # its unit tests
T=$TMPDIR/editor sh examples/editor/tools/smoke.sh   # editor: builds it, runs --smoke
sh examples/chip8/tools/smoke.sh            # chip8: golden hashes, seeded replay, window loop
sh examples/imageview/tools/smoke.sh        # imageview: decode, workers, input, export
sh examples/chat/tools/smoke.sh             # chat: rooms, DMs, bot, recovery, limiter, load, SMTP, TLS
(cd examples/chat && jacket test)           # its unit tests
```

The 2D game needs a display and OpenGL, the 3D game a WebGPU-capable GPU (no window), the server `initdb`/`pg_ctl`/`openssl` (`examples/game2d/tools/local_postgres.sh` starts a throwaway cluster).

## How to change it

Add an app as `examples/<name>/` with its own smoke script and `FRICTION.md`, then add a line to `just dogfood`. Fix compiler or stdlib problems with a test; record anything larger in `FRICTION.md`.

## Configuration

- `JAIC`: compiler to use. `JAIC_NATIVE_LIBS`: native library directory.
- `DATABASE_URL`: server database; `WITH_GAME=1` also runs the game smoke against it.
- `SERVER_URL`, `SERVER_ROOTS`: point the game smoke at a running backend (and the certificate to trust).
- game3d: `FRAME` (PNG to keep), `REQUIRE_GPU=1` (fail without a GPU), `THREADS` (default 3).
