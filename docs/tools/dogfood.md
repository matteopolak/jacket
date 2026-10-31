# Dogfood applications

## What it is

Real applications written in Jai and built with `jaic`, kept under `examples/` to exercise the compiler, stdlib and extensions together. The first is `examples/game2d/`: a 2D shooter plus a JSON backend. The second is `examples/game3d/`: a WebGPU scene with a job system, meshoptimizer meshes and an intro video. Each app keeps a `FRICTION.md` of rough edges found while writing it. They are not in CI and not listed in `tests/corpus/examples.json`.

## How it works

`examples/game2d/` holds four jacket programs and one shared module:

- `shared/`: API types, route constants, the typed `Api` client and the enemy table, used by both sides.
- `server/`: Http + Postgres + Tls + Json backend (accounts, matches, leaderboard). Args CLI: `--listen`, `--database`, `--threads`, `--pool-size`, `--tls-cert`/`--tls-key`.
- `loadtest/`: Io fibers across scheduler threads; reports throughput and p50/p99/max, exits 1 on any failure.
- `game/`: Simp/Window_Creation/Input/Gamepad/Sound_Player game with a zip asset pack, hot reload (File_Watcher), lz4 saves and a `--smoke` / `--autoplay N` mode that runs headless-ish.
- `examples/game2d/tools/assetgen/`: generates sprites, font atlas and sounds, and writes the zip pack.

`examples/game3d/` is one program (`game/main.jai`) with a smoke script:

- `world.jai` keeps entities in a `Soa` and updates them four lanes at a time with `Extensions/SIMD`; `jobs.jai` is a work queue on `Thread` and `Atomics`; `cull_job` turns entities into instance matrices in parallel.
- `meshes.jai` makes cube, sphere, torus and floor meshes and runs them through meshoptimizer (remap, cache, overdraw, fetch order, simplified level of detail, compressed round trip).
- `render.jai` draws with `Extensions/WebGPU` into a window or an offscreen texture; `intro.jai` plays an MPEG-1 clip with `pl_mpeg`; `profile.jai` times the frame phases with `Iprof`.
- `--smoke` runs headless: it compares threads against one thread, decodes the clip, renders frames offscreen and reads them back. See `examples/game3d/README.md`.

The 2D game draws in logical units; Simp sizes the framebuffer in device pixels (see `pixel_scale` in `Window_Info`), and pixel-art textures use `texture_set_filter(.., .NEAREST)`.

## Running

```
just dogfood                                # server, load test (HTTP and HTTPS), game smoke
sh examples/game2d/tools/smoke_server.sh    # backend only
sh examples/game2d/tools/smoke_game.sh      # game only; FRAME=out.png keeps the offscreen frame
sh examples/game3d/tools/smoke_game.sh      # 3D game; FRAME=out.png keeps the last offscreen frame
```

Needs a display and OpenGL for the 2D game, a WebGPU-capable GPU (no window) for the 3D one, and `initdb`/`pg_ctl`/`openssl` for the server (`examples/game2d/tools/local_postgres.sh` starts a throwaway cluster).

## How to change it

Add a new app as `examples/<name>/` with its own smoke script and `FRICTION.md`, then add a recipe line to `just dogfood`. Fix compiler/stdlib problems in the stdlib or compiler with a test; record anything larger in the app's `FRICTION.md`.

## Configuration

- `JAIC`: compiler to use (default `jaic`).
- `DATABASE_URL`: server database; `WITH_GAME=1` makes `smoke_server.sh` also run the game smoke against it.
- `SERVER_URL`, `SERVER_ROOTS`: point the game smoke at a running backend (and the certificate to trust).
- `JAIC_NATIVE_LIBS`: native library directory (mbedtls for Tls; wgpu-native, meshoptimizer and pl_mpeg for game3d).
- game3d smoke: `FRAME` (PNG to keep), `REQUIRE_GPU=1` (fail without a GPU instead of skipping the drawing checks), `THREADS` (worker threads, default 3).

## Dependencies

Stdlib (game3d): Soa, Math, Thread, Atomics, Iprof, pl_mpeg, meshoptimizer, PCG, Input, Window_Creation. Extensions (game3d): WebGPU, SIMD, Assets, Image, Args.

Stdlib (game2d): Simp, Window_Creation, Input, Gamepad, Sound_Player, stb_vorbis, Wav_File, File_Watcher, Zip_File_Directory, lz4, xxHash, PCG, Bucket_Array, Pool. Extensions: Http, Postgres, Tls, Io, Json, Args, Deflate.
