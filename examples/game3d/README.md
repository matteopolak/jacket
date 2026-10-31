# game3d

A small 3D scene written to use many standard-library and extension modules together. A few
hundred shapes drift around an arena while you fly a camera through them. It plays a short intro
video first.

```
jaic examples/game3d/game/main.jai -exe game3d
./game3d                    # intro, then the scene
./game3d --no-intro         # straight to the scene
./game3d --entities 1024    # more shapes
./game3d --threads 0        # everything on the main thread
```

Keys: `W` `A` `S` `D` move, `Q` and `E` go down and up, `Shift` is faster, the arrow keys or the
right mouse button turn, `P` pauses the simulation, `R` prints the profile, `Space` skips the
intro, `Escape` quits.

It needs a GPU that WebGPU can use: Metal on macOS, Vulkan on Linux, Direct3D 12 on Windows. Only
macOS has been run so far.

## Smoke test

```
examples/game3d/tools/smoke_game.sh
FRAME=frame.png examples/game3d/tools/smoke_game.sh   # also keep the last offscreen frame
REQUIRE_GPU=1 examples/game3d/tools/smoke_game.sh     # fail instead of skipping without a GPU
```

The script builds the game with the compiler's `+Performance_Report` plugin (it prints the first
lines of the report) and runs `game3d --smoke`: no window, no input. The session checks that

- every mesh lost duplicate vertices, got a better vertex-cache ratio, has a coarser level of detail
  and survives meshoptimizer's compressed form unchanged;
- 240 ticks of 768 entities on worker threads give exactly the same world as the same ticks on
  one thread, and 300 back-to-back jobs visit every item once;
- the intro clip decodes to 50 frames of 160 by 96 pixels that change from first to last;
- 30 frames drawn offscreen at 640 by 360 produce a frame that is not blank, draws the same thing
  twice, and is read back from the GPU; the Iprof table lists every frame phase; the intro clip
  drawn into a texture is coloured in the middle and black in the letterbox.

Without a GPU adapter the drawing checks are skipped and the rest still runs. The compiled
libraries come from `target/native-libs`; in a checkout without it, point `JAIC_NATIVE_LIBS` at a
directory that has wgpu-native, meshoptimizer and pl_mpeg.

## How it is built

| File | What it does |
| --- | --- |
| `game/main.jai` | Imports, command line, entry point. |
| `game/game.jai` | The `Game` value, the fixed-step tick, the window loop and the intro loop. |
| `game/world.jai` | Entities in a `Soa`, the SIMD update, the culling job, frustum planes. |
| `game/jobs.jai` | The job system: worker threads, a semaphore and atomic chunk counters. |
| `game/meshes.jai` | Cube, sphere, torus and floor meshes through meshoptimizer. |
| `game/render.jai` | WebGPU device, pipelines, buffers, drawing, offscreen readback. |
| `game/intro.jai` | pl_mpeg decoding into a texture and the letterboxed draw. |
| `game/camera.jai` | The camera, keys and mouse, the scripted flight. |
| `game/profile.jai` | Iprof zones for the frame phases. |
| `game/smoke.jai` | The scripted session. |
| `game/assets/intro.mpg` | The intro clip. |

A frame goes: read input, run the simulation ticks (`update_job`, spread over the workers),
cull every entity against the camera in parallel (`cull_job`, which also writes each model
matrix), pack the visible instances into one buffer grouped by mesh and level of detail, then
draw one instanced call per group.

The meshes are made as triangle soup. meshoptimizer merges the duplicate vertices, reorders the
triangles for the vertex cache, overdraw and vertex fetch, and simplifies the sphere and torus for
the far level of detail. Entities farther than `LOD_DISTANCE` use the coarse level.

## The intro clip

`game/assets/intro.mpg` is 160 by 96 pixels, 2 seconds at 25 frames per second, MPEG-1 video
only, 108 KB. It is a Mandelbrot zoom from a generator in ffmpeg, so there is no third-party
content in it:

```
ffmpeg -f lavfi -i "mandelbrot=size=160x96:rate=25:end_scale=0.05" -t 2 -an \
    -c:v mpeg1video -b:v 250k -g 12 -bf 0 -f mpeg intro.mpg
```

## Changing it

- A new mesh: add a `*_soup` function in `meshes.jai`, a `Mesh_Kind` and its radius in
  `build_meshes`. Entities pick a kind in `world_init`.
- A new per-entity field: add it to `Entity`. The `Soa` makes it its own array.
- More jobs: write a `Job_Proc` and pass it to `run_job`; each call may only touch the items of
  its own range.
- Profile zones: add a name to `Zone` and `zone_names` in `profile.jai`. Zones may only be opened
  on the main thread.

Friction found while writing it is in `FRICTION.md`.
