"""Configure the pinned sdl2-compat Unix public header, without building SDL.

The verified archive supplies every declaration and configuration constant.
Upstream's default SDL2COMPAT_X11=ON means SDL2COMPAT_DISABLE_X11=OFF.
No installed SDL2/SDL3 header is substituted into this versioned SDK.
"""

from pathlib import Path
import re
import subprocess


def prepare_runtime_headers(source, output):
    """Return generated-first include roots for the complete SDL2 runtime oracle.

    CMake script mode performs upstream configure_file on its real template.
    Generated headers use explicit LF bytes on every host; input bytes stay intact.
    Reject new configuration inputs rather than silently guessing their values.
    The caller must first validate/extract the complete hash-pinned archive.
    """
    source = Path(source).resolve(strict=True)
    output = Path(output).resolve()
    headers = source / "include/SDL2"
    template = headers / "SDL_config_unix.h.cmake"
    text = template.read_text(encoding="utf-8")
    directives = re.findall(
        r"^\s*#\s*cmakedefine(?:01)?\s+(\w+).*$", text, re.MULTILINE
    )
    if directives != ["SDL2COMPAT_DISABLE_X11"] or re.search(
        r"@[A-Za-z_][A-Za-z_0-9]*@|\$\{", text
    ):
        raise RuntimeError("Unreviewed SDL2 Unix header configuration inputs")
    cmake = (source / "CMakeLists.txt").read_text(encoding="utf-8")
    if not re.search(r'option\(SDL2COMPAT_X11\s+"Enable X11 support"\s+ON\)', cmake):
        raise RuntimeError(
            "Pinned SDL2 X11 default changed; review header configuration"
        )
    output.mkdir(parents=True, exist_ok=True)
    generated = output / "SDL_config_unix.h"

    # Bracket arguments preserve spaces, backslashes and semicolons literally.
    def quote(path):
        value = Path(path).as_posix()
        delimiter = "="
        while "]" + delimiter + "]" in value:
            delimiter += "="
        return "[" + delimiter + "[" + value + "]" + delimiter + "]"

    script = output / "configure.cmake"
    script.write_text(
        "set(SDL2COMPAT_DISABLE_X11 OFF)\nconfigure_file("
        + quote(template)
        + " "
        + quote(generated)
        + " NEWLINE_STYLE LF)\n",
        encoding="utf-8",
    )
    # Never accept a stale generated header after a failed configuration.
    generated.unlink(missing_ok=True)
    subprocess.run(["cmake", "-P", str(script)], check=True)
    if not generated.is_file():
        raise RuntimeError("CMake did not generate the SDL2 Unix SDK header")
    return [output, headers]
