"""Discover an installed OpenSSL development SDK, then verify its real C ABI."""

import argparse
from dataclasses import asdict, dataclass
import json
import os
from pathlib import Path
import platform
import shlex
import shutil
import subprocess
import sysconfig


class SDKError(RuntimeError):
    pass


@dataclass(frozen=True)
class OpenSSLSDK:
    prefix: Path
    include_dir: Path
    library: Path
    cflags: tuple[str, ...]
    ldflags: tuple[str, ...]
    source: str

    def runtime_environment(self, environ=None):
        env = dict(os.environ if environ is None else environ)
        if os.name == "nt":
            directories = [self.prefix / "bin", self.library.parent]
            env["PATH"] = os.pathsep.join(map(str, directories)) + os.pathsep + env.get("PATH", "")
        return env


def _pkg_config(option, env):
    executable = env.get("PKG_CONFIG") or shutil.which("pkg-config", path=env.get("PATH"))
    if not executable:
        return None
    try:
        result = subprocess.run([executable, option, "libcrypto"], env=env,
                                capture_output=True, text=True, timeout=15)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return result.stdout.strip() if result.returncode == 0 else None


def _brew_prefix(env):
    if platform.system() != "Darwin":
        return None
    brew = shutil.which("brew", path=env.get("PATH"))
    if not brew:
        brew = next((str(path) for path in (Path("/opt/homebrew/bin/brew"), Path("/usr/local/bin/brew")) if path.is_file()), None)
    if not brew:
        return None
    try:
        result = subprocess.run([brew, "--prefix", "openssl@3"], env=env,
                                capture_output=True, text=True, timeout=15)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return Path(result.stdout.strip()) if result.returncode == 0 and result.stdout.strip() else None


def _system_prefixes(env):
    if platform.system() == "Windows":
        roots = [env.get("ProgramFiles"), env.get("ProgramW6432")]
        return [Path(root) / name for root in roots if root for name in ("OpenSSL-Win64", "OpenSSL")]
    return [Path("/usr/local"), Path("/usr")]


def _from_prefix(prefix, source, include_dir=None, library_dir=None, cflags=(), ldflags=()):
    prefix = Path(prefix).expanduser().resolve()
    include = Path(include_dir).resolve() if include_dir else prefix / "include"
    if not (include / "openssl/opensslv.h").is_file():
        raise SDKError(f"OpenSSL SDK is missing include/openssl/opensslv.h: {include}")
    directories = [Path(library_dir).resolve()] if library_dir else []
    if not library_dir:
        multiarch = sysconfig.get_config_var("MULTIARCH")
        if multiarch:
            directories.append(prefix / "lib" / multiarch)
        architecture = "arm64" if platform.machine().lower() in ("arm64", "aarch64") else "x64"
        directories += [prefix / "lib", prefix / "lib64", prefix / "lib/VC" / architecture / "MD", prefix / "lib/VC" / architecture / "MT"]
    names = ("libcrypto.dylib", "libcrypto.so", "libcrypto.lib", "crypto.lib", "libcrypto.a")
    library = next((directory / name for directory in directories for name in names if (directory / name).is_file()), None)
    if library is None:
        raise SDKError(f"OpenSSL development SDK has no linkable libcrypto library: {prefix}")
    compile_flags = tuple(dict.fromkeys(["-I" + str(include), *cflags]))
    # Select this installation even when pkg-config filters system -L/-I flags.
    link_flags = [str(library), *(flag for flag in ldflags if flag != "-lcrypto")]
    if library.suffix in (".so", ".dylib") and platform.system() != "Windows":
        link_flags.append("-Wl,-rpath," + str(library.parent))
    return OpenSSLSDK(prefix, include, library, compile_flags, tuple(link_flags), source)


def discover_sdk(prefix=None, *, environ=None):
    """Explicit CLI/env selection is authoritative; missing SDKs always fail."""
    env = dict(os.environ if environ is None else environ)
    explicit = prefix or env.get("OPENSSL_ROOT_DIR") or env.get("OPENSSL_DIR")
    if explicit:
        return _from_prefix(explicit, "explicit")
    installed_prefix = _pkg_config("--variable=prefix", env)
    if installed_prefix:
        include = _pkg_config("--variable=includedir", env)
        libdir = _pkg_config("--variable=libdir", env)
        cflags = _pkg_config("--cflags", env)
        ldflags = _pkg_config("--libs", env)
        if not include or not libdir or cflags is None or ldflags is None:
            raise SDKError("Installed libcrypto pkg-config metadata is incomplete")
        return _from_prefix(installed_prefix, "pkg-config:libcrypto", include, libdir,
                            shlex.split(cflags), shlex.split(ldflags))
    brew = _brew_prefix(env)
    candidates = ([brew] if brew else []) + _system_prefixes(env)
    errors = []
    for candidate in candidates:
        try:
            return _from_prefix(candidate, "installed-prefix")
        except SDKError as error:
            errors.append(str(error))
    raise SDKError("OpenSSL development SDK not found. Install the development headers/libraries and pkg-config, "
                   "or set OPENSSL_ROOT_DIR/OPENSSL_DIR to a complete SDK. " + "; ".join(errors))


def probe_sdk(sdk, output, *, compiler=None):
    """Compile/link/run with the selected headers and library; never waive a host."""
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    scratch = output / "scratch"
    scratch.mkdir(exist_ok=True)
    env = sdk.runtime_environment()
    env.update(TMPDIR=str(scratch), TEMP=str(scratch), TMP=str(scratch))
    command = list(compiler) if compiler else shlex.split(env.get("CC", "cc"))
    source = output / "openssl-sdk-probe.c"
    executable = output / ("openssl-sdk-probe.exe" if os.name == "nt" else "openssl-sdk-probe")
    source.write_text(
        '#include <openssl/opensslv.h>\n#include <openssl/crypto.h>\n#include <stdio.h>\n'
        '#if !defined(OPENSSL_VERSION_MAJOR) || OPENSSL_VERSION_MAJOR < 3\n'
        '#error OpenSSL_3_development_SDK_required\n#endif\n'
        'int main(void) { unsigned long runtime = OpenSSL_version_num();\n'
        ' printf("%s\\n%lu\\n%lu\\n", OPENSSL_VERSION_TEXT, (unsigned long)OPENSSL_VERSION_NUMBER, runtime);\n'
        ' return (runtime >> 28) != ((unsigned long)OPENSSL_VERSION_NUMBER >> 28); }\n'
    )
    msvc = Path(command[0]).name.lower() in ("cl", "cl.exe")
    if msvc:
        arguments = [*command, "/nologo", "/W4", "/WX", "/I" + str(sdk.include_dir), str(source), "/Fe:" + str(executable), "/link", str(sdk.library)]
    else:
        arguments = [*command, "-std=c11", "-Wall", "-Wextra", "-Werror", *sdk.cflags, str(source), *sdk.ldflags, "-o", str(executable)]
    try:
        built = subprocess.run(arguments, cwd=output, env=env, capture_output=True, text=True, timeout=60)
        (output / "compile.log").write_text(built.stdout + built.stderr)
        if built.returncode:
            raise SDKError(f"OpenSSL SDK compile/link probe failed: {built.stdout}{built.stderr}")
        ran = subprocess.run([str(executable)], cwd=output, env=env, capture_output=True, text=True, timeout=30)
        (output / "run.log").write_text(ran.stdout + ran.stderr)
        lines = ran.stdout.splitlines()
        if ran.returncode or ran.stderr or len(lines) != 3:
            raise SDKError(f"OpenSSL SDK runtime probe failed: {ran.stdout}{ran.stderr}")
        header, runtime = int(lines[1]), int(lines[2])
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        raise SDKError(f"OpenSSL SDK probe could not complete: {error}") from error
    receipt = dict(header_version=lines[0], header_major=header >> 28, runtime_major=runtime >> 28,
                   compiler_command=arguments, sdk={key: str(value) if isinstance(value, Path) else value for key, value in asdict(sdk).items()})
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--openssl", type=Path)
    parser.add_argument("--probe", action="store_true")
    args = parser.parse_args()
    try:
        sdk = discover_sdk(args.openssl)
        if args.probe:
            target = os.environ.get("CARGO_TARGET_DIR")
            if not target:
                raise SDKError("Set CARGO_TARGET_DIR for probe artifacts; no local output fallback is used")
            value = probe_sdk(sdk, Path(target) / "openssl-sdk-verification")
        else:
            value = {key: str(item) if isinstance(item, Path) else item for key, item in asdict(sdk).items()}
        print(json.dumps(value, indent=2))
    except SDKError as error:
        parser.exit(1, str(error) + "\n")


if __name__ == "__main__":
    main()
