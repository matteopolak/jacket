"""The platform the tools are running on, named the way the repo names target directories."""
import platform

_SYSTEMS = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}
_CPUS = {"arm64": "arm64", "aarch64": "arm64", "amd64": "x64", "x86_64": "x64"}


def host_platform() -> str:
    """`<os>-<cpu>` of this machine, e.g. `macos-arm64`, `linux-x64`, `windows-x64`."""
    system = _SYSTEMS.get(platform.system())
    cpu = _CPUS.get(platform.machine().lower())
    if not system or not cpu:
        raise ValueError(f"unsupported host {platform.system()} {platform.machine()}")
    return f"{system}-{cpu}"
