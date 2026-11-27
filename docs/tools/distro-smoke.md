# Distro smoke tests

`tools/dev/distro_smoke.sh` installs a jacket release in a clean container for each of Ubuntu, Debian, Fedora, openSUSE Tumbleweed and Arch Linux, then runs the commands a new user runs first. It checks that a release works on each distribution's own C compiler, glibc and file layout. It runs on a Mac with Apple's `container` tool, and is not part of CI. The CI checks of each install channel per OS are the [release smoke tests](release-smoke.md).

```sh
tools/dev/distro_smoke.sh                       # every distro, latest release
JAI_VERSION=0.40.0 tools/dev/distro_smoke.sh arch fedora
tools/dev/distro_smoke.sh --list
```

```
ubuntu     ok
debian     ok
fedora     ok
opensuse   ok
arch       FAILED: jacket test
           log: target/distro-smoke/arch.log
```

## How it works

- Each distro gets a fresh container from its public image. The distro's package manager installs `gcc`, `git` and `curl`, and `install.sh` installs the release.
- `tools/dev/distro_smoke_steps.sh` runs inside the container. It runs `jaic -interp`, `jacket run`, `jacket build` and the built program, `jacket new` and `jacket run`, and a `jacket new --lib` project with a test and a doc example under `jacket test`. Then it runs `jacket fmt --check`, `jacket lint` and `jacket toolchain`.
- Each step prints `smoke: STEP`. The first failure prints `smoke: FAILED STEP` and stops that container. The script prints one line per distro and exits 1 if any failed.

The containers are arm64 by default, so they run natively on Apple Silicon. `SMOKE_ARCH=amd64` runs x86-64 images under Rosetta. Under Rosetta, Arch's glibc fails `posix_spawn` with ENOMEM once a process has a few 1 GiB thread stacks, which jaic's worker pool has. The linker then cannot start. Native arm64 machines with the same glibc version spawn normally. Treat an amd64-only spawn failure as Rosetta's until it shows up on real hardware.

## How to change it

- **Add a distro:** add a `name|image|setup command` line to `distros` in `distro_smoke.sh`. The setup command must install a C compiler, `git` and `curl`.
- **Add a step:** add a `step "name" command...` line to `distro_smoke_steps.sh`. Use `expect_hello` for a command whose output must be the hello line.
- **Test a release before it is published:** put its archives and `SHA256SUMS` in `DIR/v<version>/`, then run `JAI_BASE_URL=file://DIR JAI_VERSION=<version> tools/dev/distro_smoke.sh`. The folder is mounted read-only into each container.

## Configuration

| Variable | Effect |
| --- | --- |
| `JAI_VERSION` | the release to install (default: the latest) |
| `JAI_BASE_URL` | where `install.sh` fetches from; a `file://` folder is mounted into the container |
| `SMOKE_ARCH` | `arm64` (default) or `amd64` |
| `SMOKE_MEMORY`, `SMOKE_CPUS` | container size (default `4G` and `4`) |
| `SMOKE_KEEP=1` | keep the logs of passing distros in `target/distro-smoke` too |

## Dependencies

- Apple's `container` tool, started with `container system start`.
- The network, to pull images and fetch packages and the release.
- `install.sh`, for installing the release.
