# Dogfood applications

## What it is

Real applications written in Jai and built with `jaic`, kept under `examples/` to exercise the compiler, stdlib and extensions together. The first is `examples/game2d/`: a 2D shooter plus a JSON backend. Each app keeps a `FRICTION.md` of rough edges found while writing it. They are not in CI and not listed in `tests/corpus/examples.json`.

## How it works

`examples/game2d/` holds four jacket programs and one shared module:

- `shared/`: API types, route constants, the typed `Api` client and the enemy table, used by both sides.
- `server/`: Http + Postgres + Tls + Json backend (accounts, matches, leaderboard). Args CLI: `--listen`, `--database`, `--threads`, `--pool-size`, `--tls-cert`/`--tls-key`.
- `loadtest/`: Io fibers across scheduler threads; reports throughput and p50/p99/max, exits 1 on any failure.
- `game/`: Simp/Window_Creation/Input/Gamepad/Sound_Player game with a zip asset pack, hot reload (File_Watcher), lz4 saves and a `--smoke` / `--autoplay N` mode that runs headless-ish.
- `examples/game2d/tools/assetgen/`: generates sprites, font atlas and sounds, and writes the zip pack.

The game draws in logical units; Simp sizes the framebuffer in device pixels (see `pixel_scale` in `Window_Info`), and pixel-art textures use `texture_set_filter(.., .NEAREST)`.

## Running

```
just dogfood                                # server, load test (HTTP and HTTPS), game smoke
sh examples/game2d/tools/smoke_server.sh    # backend only
sh examples/game2d/tools/smoke_game.sh      # game only; FRAME=out.png keeps the offscreen frame
```

Needs a display and OpenGL for the game, and `initdb`/`pg_ctl`/`openssl` for the server (`examples/game2d/tools/local_postgres.sh` starts a throwaway cluster).

## How to change it

Add a new app as `examples/<name>/` with its own smoke script and `FRICTION.md`, then add a recipe line to `just dogfood`. Fix compiler/stdlib problems in the stdlib or compiler with a test; record anything larger in the app's `FRICTION.md`.

## Configuration

- `JAIC`: compiler to use (default `jaic`).
- `DATABASE_URL`: server database; `WITH_GAME=1` makes `smoke_server.sh` also run the game smoke against it.
- `SERVER_URL`, `SERVER_ROOTS`: point the game smoke at a running backend (and the certificate to trust).
- `JAIC_NATIVE_LIBS`: native library directory (mbedtls for Tls).

## Dependencies

Stdlib: Simp, Window_Creation, Input, Gamepad, Sound_Player, stb_vorbis, Wav_File, File_Watcher, Zip_File_Directory, lz4, xxHash, PCG, Bucket_Array, Pool. Extensions: Http, Postgres, Tls, Io, Json, Args, Deflate.
