# Dogfood applications

Real applications written in Jai under `examples/`, to exercise the compiler, stdlib and extensions together. Each keeps a `FRICTION.md` of rough edges found while writing it. They are not in CI.

- `examples/game2d/`: a 2D shooter plus a JSON backend. `shared/` (API types, typed client), `server/` (Http, Postgres, Tls, Json; `--listen`, `--database`, `--threads`, `--pool-size`, `--tls-cert`/`--tls-key`), `loadtest/` (Io fibers; reports throughput and p50/p99/max), `game/` (Simp, window, input, sound, zip asset pack, hot reload, `--smoke` and `--autoplay N`), and `examples/game2d/tools/assetgen/`.
- `examples/game3d/`: one program (`game/main.jai`): `Soa` entities updated with `Extensions/SIMD`, a job system on `Thread` and `Atomics`, meshoptimizer meshes, WebGPU rendering, an MPEG-1 intro via `pl_mpeg`. See its README.

## Running

```sh
just dogfood                                # server, load test (HTTP and HTTPS), game smoke
sh examples/game2d/tools/smoke_server.sh    # backend only
sh examples/game2d/tools/smoke_game.sh      # game only; FRAME=out.png keeps the offscreen frame
sh examples/game3d/tools/smoke_game.sh      # 3D game
```

The 2D game needs a display and OpenGL, the 3D game a WebGPU-capable GPU (no window), the server `initdb`/`pg_ctl`/`openssl` (`examples/game2d/tools/local_postgres.sh` starts a throwaway cluster).

## How to change it

Add an app as `examples/<name>/` with its own smoke script and `FRICTION.md`, then add a line to `just dogfood`. Fix compiler or stdlib problems with a test; record anything larger in `FRICTION.md`.

## Configuration

- `JAIC`: compiler to use. `JAIC_NATIVE_LIBS`: native library directory.
- `DATABASE_URL`: server database; `WITH_GAME=1` also runs the game smoke against it.
- `SERVER_URL`, `SERVER_ROOTS`: point the game smoke at a running backend (and the certificate to trust).
- game3d: `FRAME` (PNG to keep), `REQUIRE_GPU=1` (fail without a GPU), `THREADS` (default 3).
