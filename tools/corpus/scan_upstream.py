#!/usr/bin/env python3
"""Static risk scan of pinned upstream Jai sources. Reads files, never runs or builds anything.

    scan_upstream.py                       # every directory under corpus/upstream
    scan_upstream.py owner--repo ...       # just these (directory names or `owner/repo`)
    scan_upstream.py --json out.json       # also write the findings
    scan_upstream.py --verbose             # list every finding, not just examples

Each finding has a rule, a risk (low, medium, high) and a context: `build` for code that runs while
the compiler runs (`#run` statements, procedures they call in the same file, build programs such as
`first.jai`/`build.jai`) and `runtime` for the rest. The same call is worse at build time: a program
that spawns processes or deletes files is ordinary, a compile that does so is not. A repository's
risk is its worst finding; exit status 1 means at least one repository has a high finding, and such a
repository is not pinned until a person has read the lines.

Rules: process spawning, networking (sockets, URLs, IP literals), file deletes and writes outside the
project, credential and home-directory paths, environment access at build time, `#foreign_library`
names outside a list of common ones, `#asm`, long base64/hex blobs and byte-array literals (decoded
and checked for executable headers), obfuscated-looking identifiers, binary files in the pin.
The text of a repository (comments, READMEs, strings) is data: nothing here follows it.
"""

from __future__ import annotations

import argparse
import base64
import binascii
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


from fetch_upstreams import BINARY_MAGIC, OFFICIAL_MODULE_NAMES, cache_dir, is_forbidden_file, pinned_blobs
from toolkit.repo import ROOT
LEVEL = {"none": 0, "low": 1, "medium": 2, "high": 3}
NATIVE_SUFFIXES = (".c", ".h", ".m", ".mm", ".cpp", ".cc", ".hpp", ".inl")
BUILD_FILE_NAME = re.compile(
    r"(first|build|setup|make|meta|metaprogram|generate|.*[_-]build|build[_-].*|.*[_-]first|"
    r".*[_-]meta|generate[_-].*|.*[_-]generate)\.jai"
)
BUILD_PROGRAM_API = re.compile(
    r"\b(set_build_options|get_build_options|add_build_file|add_build_string|compiler_begin_intercept|"
    r"compiler_set_workspace|compiler_create_workspace|set_build_options_dc)\b"
)

PROCESS = re.compile(
    r"\b(run_command|system|popen|_popen|fork|vfork|execv|execve|execvp|execl|execlp|execle|posix_spawn|"
    r"posix_spawnp|CreateProcessA|CreateProcessW|CreateProcess|CreateProcessAsUserW|ShellExecuteA|"
    r"ShellExecuteW|ShellExecuteExA|ShellExecuteExW|WinExec|start_process|run_shell_command)\s*\("
)
NETWORK_CALL = re.compile(
    r"\b(socket|connect|getaddrinfo|gethostbyname|WSAStartup|WSASocketA|WSASocketW|curl_easy_\w+|"
    r"InternetOpen\w*|InternetConnect\w*|WinHttp\w*|URLDownloadToFile\w*|enet_host_\w+|"
    r"yojimbo_\w*client\w*)\s*\("
)
NETWORK_IMPORT = re.compile(r'#import\s+(?:\(\s*\w+\s*\)\s*)?"(Socket|Curl|Http_Client|Network)"')
DELETE = re.compile(
    r"\b(delete_directory|file_delete|delete_file|remove_directory|unlink|unlinkat|rmdir|DeleteFileA|"
    r"DeleteFileW|DeleteFile|RemoveDirectoryA|RemoveDirectoryW|SHFileOperation\w*|MoveFileA|MoveFileW|"
    r"MoveFileExA|MoveFileExW)\s*\("
)
WRITE = re.compile(
    r"\b(write_entire_file|copy_file|make_directory_if_it_does_not_exist|fopen|fwrite|CreateFileA|"
    r"CreateFileW|WriteFile)\s*\(|\bfile_open\s*\([^\n]*\bfor_writing\b|\bfor_writing\b"
)
OUTSIDE_PATH = re.compile(
    r'"(?:/(?:etc|usr|home|Users|var|tmp|bin|sbin|opt|root|Library|System|private|mnt|Volumes|proc|dev|boot)\b'
    r'|[A-Za-z]:[\\/]|~[/\\"]|\.\.[/\\"]|\\\\\\\\)'
)
SENSITIVE_PATH = re.compile(
    r"(\.ssh\b|id_rsa|id_ed25519|id_ecdsa|authorized_keys|known_hosts|\.aws\b|\.gnupg|\.netrc|\.npmrc|"
    r"\.pypirc|\.kube\b|\.git-credentials|/etc/passwd|/etc/shadow|wallet\.dat|Login Data|"
    r"Library/Keychains|Application Support/Google/Chrome|Mozilla/Firefox/Profiles|\.config/gcloud|"
    r"\.docker/config|keychain-db|SSH_AUTH_SOCK)",
    re.I,
)
ENVIRONMENT = re.compile(
    r"\b(get_environment_variable|set_environment_variable|getenv|setenv|putenv|GetEnvironmentVariable\w*|"
    r"SetEnvironmentVariable\w*|get_home_directory|get_user_directory|SHGetKnownFolderPath|"
    r"SHGetFolderPath\w*|GetUserProfileDirectory\w*)\s*\(|\"(?:HOME|USERPROFILE|APPDATA|LOCALAPPDATA)\""
)
STRING = re.compile(r'"((?:\\.|[^"\\])*)"')
SECRETISH = re.compile(r"\b(password|passwd|credential|api_?key|secret|auth_?token|access_?token)s?\b", re.I)
URL = re.compile(r"https?://[^\s\"']+")
IP = re.compile(r"(?<![\d.])(?:\d{1,3}\.){3}\d{1,3}(?![\d.])")
FOREIGN = re.compile(r'#(?:foreign_system_library|foreign_library|system_library|library)\b[^"\n;]*"([^"]+)"')
DIRECTIVE_LINE = re.compile(r"#(?:import|load|foreign_library|foreign_system_library|library|system_library)\b")
ASM = re.compile(r"#asm\b")
LOAD_LIBRARY = re.compile(r"\b(LoadLibrary[AW]?|LoadLibraryEx[AW]?|dlopen)\s*\(")
NATIVE_PROCESS = re.compile(
    r"\b(system|popen|_popen|fork|execv|execve|execvp|execl|execlp|posix_spawn|CreateProcess[AW]?|"
    r"ShellExecute[AW]?|WinExec)\s*\("
)
NATIVE_NETWORK = re.compile(r"\b(socket|connect|getaddrinfo|WSAStartup|curl_easy_init|InternetOpen\w*)\s*\(")
BASE64_RUN = re.compile(r"[A-Za-z0-9+/_-]{256,}={0,2}")
HEX_RUN = re.compile(r"(?:[0-9a-fA-F]{2}[ ,:\\x]*){128,}")
ARRAY_LITERAL = re.compile(r"\.\[")
OBFUSCATED = [
    re.compile(r"\b_0x[0-9a-fA-F]{4,}\b"),
    re.compile(r"\b[Il1]{10,}\b"),
    re.compile(r"\b[Oo0]{10,}\b"),
    re.compile(r"\b[A-Za-z_]\w{0,3}[0-9a-f]{24,}\b"),
]
IDENTIFIER = re.compile(r"\b[A-Za-z_]\w{13,}\b")
CALL_NAME = re.compile(r"\b([A-Za-z_]\w*)\s*\(")

# Libraries that ordinary programs link; anything else named by a #foreign_library is reported.
COMMON_LIBRARIES = frozenset(
    """c libc m dl pthread rt util gcc gcc_s stdc++ c++ msvcrt ucrt vcruntime msvcprt legacy_stdio_definitions
    kernel32 user32 gdi32 shell32 ole32 oleaut32 opengl32 glu32 advapi32 comdlg32 comctl32 winmm imm32
    dwmapi shlwapi uxtheme version dbghelp psapi setupapi hid dinput8 dxguid xinput xinput1_4 xinput9_1_0
    d3d11 d3d12 dxgi d3dcompiler dxcompiler dwrite d2d1 windowscodecs mf mfplat bcrypt crypt32 secur32
    cabinet userenv wtsapi32 propsys cfgmgr32 netapi32 mpr rpcrt4 comsuppw ntdll
    GL GLX GLU EGL GLESv2 X11 Xext Xi Xcursor Xrandr Xinerama Xfixes Xrender xcb wayland-client wayland-egl
    wayland-cursor xkbcommon vulkan vulkan-1 MoltenVK glfw glfw3 SDL SDL2 SDL3 freetype harfbuzz
    stb_image miniaudio raylib raygui openal OpenAL32 portaudio asound pulse jack alsa udev dbus-1
    Cocoa AppKit Foundation Metal MetalKit QuartzCore CoreFoundation CoreGraphics CoreText CoreAudio
    AudioToolbox IOKit OpenGL Carbon Security ApplicationServices ImageIO Accelerate""".split(),
    # compared case-insensitively
)
COMMON_LIBRARIES = frozenset(name.lower() for name in COMMON_LIBRARIES)
NETWORK_LIBRARIES = {"ws2_32", "wsock32", "wininet", "winhttp", "curl", "libcurl", "mswsock", "iphlpapi", "enet", "websockets"}

SHEBANG_OR_EXE = tuple(BINARY_MAGIC) + (b"#!",)


@dataclass
class Finding:
    rule: str
    risk: str
    context: str  # "build" or "runtime"
    path: str
    line: int
    text: str


def mask(text: str) -> tuple[str, str]:
    """(comments blanked, comments and string contents blanked), both the length of `text` and with the
    same line breaks. Handles // and nested /* */ comments, "strings" and #string heredocs."""
    nocomment = list(text)
    code = list(text)

    def blank(chars: list[str], start: int, end: int) -> None:
        for k in range(start, end):
            if chars[k] != "\n":
                chars[k] = " "

    token = re.compile(r'//|/\*|"|#string[ \t]+([A-Za-z_]\w*)')
    string = re.compile(r'"(?:\\.|[^"\\])*"', re.S)
    pos = 0
    while True:
        found = token.search(text, pos)
        if not found:
            break
        start = found.start()
        what = found.group(0)
        if what == "//":
            end = text.find("\n", start)
            end = len(text) if end < 0 else end
            blank(nocomment, start, end)
            blank(code, start, end)
        elif what == "/*":
            depth, k = 1, start + 2
            while depth and k < len(text):
                if text.startswith("/*", k):
                    depth, k = depth + 1, k + 2
                elif text.startswith("*/", k):
                    depth, k = depth - 1, k + 2
                else:
                    k += 1
            end = k
            blank(nocomment, start, end)
            blank(code, start, end)
        elif what == '"':
            m = string.match(text, start)
            end = m.end() if m else start + 1
            blank(code, start + 1, max(start + 1, end - 1))
        else:  # #string DELIMITER heredoc: raw text up to the delimiter
            line_end = text.find("\n", found.end())
            line_end = len(text) if line_end < 0 else line_end
            stop = text.find(found.group(1), line_end)
            end = len(text) if stop < 0 else stop + len(found.group(1))
            blank(code, line_end, end)
        pos = max(end, start + 1)
    return "".join(nocomment), "".join(code)


def match_close(code: str, open_at: int, opener: str, closer: str) -> int:
    depth = 0
    for k in range(open_at, len(code)):
        if code[k] == opener:
            depth += 1
        elif code[k] == closer:
            depth -= 1
            if depth == 0:
                return k
    return len(code) - 1


def line_of(offsets: list[int], index: int) -> int:
    lo, hi = 0, len(offsets)
    while lo < hi:
        mid = (lo + hi) // 2
        if offsets[mid] <= index:
            lo = mid + 1
        else:
            hi = mid
    return lo  # 1-based


def build_spans(code: str, name: str) -> list[tuple[int, int]]:
    """Character spans that run at compile time: `#run` statements and the procedures they call."""
    spans: list[tuple[int, int]] = []
    if BUILD_FILE_NAME.fullmatch(name.lower()) or BUILD_PROGRAM_API.search(code):
        return [(0, len(code))]
    for m in re.finditer(r"#run\b(?:\s*,\s*\w+)*\s*", code):
        k = m.end()
        if k < len(code) and code[k] == "{":
            end = match_close(code, k, "{", "}")
        else:
            depth, end = 0, k
            while end < len(code):
                c = code[end]
                if c in "({[":
                    depth += 1
                elif c in ")}]":
                    depth -= 1
                elif c == ";" and depth <= 0:
                    break
                end += 1
        spans.append((m.start(), end))
    definitions = {}
    for m in re.finditer(r"^[ \t]*([A-Za-z_]\w*)[ \t]*::[ \t]*(?:inline[ \t]+|no_inline[ \t]+)?\(", code, re.M):
        close = match_close(code, m.end() - 1, "(", ")")
        brace = re.compile(r"[{;]").search(code, close)
        if brace and brace.group() == "{":
            definitions.setdefault(m.group(1), (m.start(), match_close(code, brace.start(), "{", "}")))
    seen: set[str] = set()
    for _ in range(4):
        grew = False
        for start, end in list(spans):
            for call in CALL_NAME.findall(code[start:end]):
                if call in definitions and call not in seen:
                    seen.add(call)
                    spans.append(definitions[call])
                    grew = True
        if not grew:
            break
    return spans


def first_literal(line: str, after: re.Match | None) -> str | None:
    """The first string literal in the call whose name matched, when it is the first argument."""
    if not after:
        return None
    m = re.match(r'\s*"((?:\\.|[^"\\])*)"', line[after.end():])
    return m.group(1) if m else None


def is_outside(path: str) -> bool:
    return bool(OUTSIDE_PATH.search('"' + path + '"'))


def decoded_header(sample: str) -> bool:
    try:
        raw = base64.b64decode(sample[:88] + "=" * (-len(sample[:88]) % 4), validate=False)
    except (binascii.Error, ValueError):
        return False
    return raw.startswith(SHEBANG_OR_EXE)


def array_literals(code: str):
    """(start, text) of each `.[ ... ]` literal with a numeric majority of at least 256 elements."""
    for m in ARRAY_LITERAL.finditer(code):
        end = match_close(code, m.end() - 1, "[", "]")
        body = code[m.end():end]
        count = body.count(",") + 1
        if count >= 256 and len(re.findall(r"(?:0x[0-9a-fA-F]+|\d+)\s*(?:,|$)", body)) * 2 >= count:
            yield m.start(), body


def scan_text(path: str, text: str, name: str | None = None) -> list[Finding]:
    """Findings for one Jai source."""
    name = name or Path(path).name
    nocomment, code = mask(text)
    spans = build_spans(code, name)
    lines_nc = nocomment.split("\n")
    lines_code = code.split("\n")
    line_start = [0]
    for ln in lines_code[:-1]:
        line_start.append(line_start[-1] + len(ln) + 1)
    build_lines: set[int] = set()
    whole = spans == [(0, len(code))]  # a build program: every line runs at compile time
    for a, b in spans:
        build_lines.update(range(line_of(line_start, a), line_of(line_start, max(a, b)) + 1))
    found: list[Finding] = []

    def add(rule: str, risk: str, number: int, shown: str, ctx: str | None = None) -> None:
        context = ctx or ("build" if number in build_lines else "runtime")
        found.append(Finding(rule, risk, context, path, number, shown.strip()[:160]))

    for index, (raw, stripped) in enumerate(zip(lines_nc, lines_code)):
        number = index + 1
        build = number in build_lines
        for m in PROCESS.finditer(stripped):
            add("process-spawn", "high" if build else "medium", number, raw)
        for m in NETWORK_CALL.finditer(stripped):
            add("network", "high" if build else "medium", number, raw)
        if NETWORK_IMPORT.search(raw):
            add("network-import", "high" if build else "medium", number, raw)
        for m in DELETE.finditer(stripped):
            literal = first_literal(raw, m)
            safe = literal is not None and not is_outside(literal) and not SENSITIVE_PATH.search(literal)
            add("file-delete", ("medium" if safe else "high") if build else "medium", number, raw)
        if WRITE.search(stripped):
            outside = OUTSIDE_PATH.search(raw)
            add("file-write", "high" if outside and build else "medium" if build else "low", number, raw)
        if OUTSIDE_PATH.search(raw) and not DIRECTIVE_LINE.search(stripped):
            touches = PROCESS.search(stripped) or DELETE.search(stripped) or WRITE.search(stripped)
            if touches:
                add("outside-path", "high" if build else "medium", number, raw)
            elif build:
                add("outside-path", "medium", number, raw)
        if SENSITIVE_PATH.search(raw):
            add("credential-path", "high", number, raw)
        if ENVIRONMENT.search(stripped) or ENVIRONMENT.search(raw):
            add("environment", "high" if build else "low", number, raw)
        if build and any(SECRETISH.search(m.group(1)) for m in STRING.finditer(raw)):
            add("secret-word", "medium", number, raw)
        for url in URL.findall(raw):
            if "localhost" not in url and "127.0.0.1" not in url:
                add("url", "high" if build else "low", number, url)
                break
        for ip in IP.findall(raw):
            octets = [int(o) for o in ip.split(".")]
            if max(octets) <= 255 and not ip.startswith(("0.", "127.")):
                private = ip.startswith(("10.", "192.168.")) or ip.startswith("172.") and 16 <= octets[1] <= 31
                add("ip-literal", "high" if build else "low" if private else "medium", number, ip)
                break
        for lib in FOREIGN.findall(raw):
            if "%" in lib or "\\n" in lib:  # a generator printing a declaration, not declaring one
                continue
            base = re.split(r"[\\/]", lib)[-1]
            base = re.sub(r"\.(dll|so(\.\d+)*|dylib|lib|a|framework)$", "", base, flags=re.I)
            base = re.sub(r"^lib", "", base).lower()
            if base in NETWORK_LIBRARIES:
                add("foreign-network-library", "high" if build else "medium", number, raw)
            elif ".." in lib or lib.startswith(("/", "~")) or re.match(r"[A-Za-z]:", lib):
                add("foreign-library-path", "high" if build else "medium", number, raw)
            elif base not in COMMON_LIBRARIES and not re.sub(r"[\d_-]+$", "", base) in COMMON_LIBRARIES:
                add("foreign-library", "medium", number, raw)
        if LOAD_LIBRARY.search(stripped):
            add("dynamic-load", "high" if build else "medium", number, raw)
        if ASM.search(stripped):
            add("asm", "high" if build else "medium", number, raw)
        for m in STRING.finditer(raw):
            body = m.group(1)
            if len(body) >= 256 and BASE64_RUN.search(body):
                bad = decoded_header(BASE64_RUN.search(body).group())
                add("blob-base64", "high" if bad or whole else "medium", number, f"{len(body)} chars" + (" decoding to an executable header" if bad else ""))
            elif len(body) >= 256 and HEX_RUN.search(body):
                add("blob-hex", "high" if whole else "medium", number, f"{len(body)} chars")
        if len(raw) > 4000:
            add("long-line", "low", number, f"{len(raw)} chars")
        if OBFUSCATED[0].search(stripped) or any(p.search(stripped) for p in OBFUSCATED[1:]):
            add("obfuscated-identifier", "medium", number, raw)
        else:
            for ident in IDENTIFIER.findall(stripped):
                if any(
                    len(part) >= 14 and part.isalpha() and not part.isupper() and sum(c in "aeiouAEIOU" for c in part) / len(part) < 0.1
                    for part in ident.split("_")
                ):
                    add("obfuscated-identifier", "medium", number, ident)
                    break
    for start, body in array_literals(code):
        number = line_of(line_start, start)
        values = re.findall(r"0x[0-9a-fA-F]+|\d+", body[:64])
        head = bytes(int(v, 0) & 255 for v in values[:4])
        bad = head.startswith(SHEBANG_OR_EXE)
        add("byte-array", "high" if bad or whole else "medium", number, f"{body.count(',') + 1} elements" + (" with an executable header" if bad else ""))
    if len(text) > 1024 * 1024:
        add("large-source", "low", 1, f"{len(text) // 1024} KiB")
    return found


def scan_native(path: str, text: str) -> list[Finding]:
    """C-family sources are vendored libraries or small helpers: process and network calls are noted
    (medium), credential paths and executable-looking data are not."""
    nocomment, code = mask(text)
    found = []
    for number, (raw, stripped) in enumerate(zip(nocomment.split("\n"), code.split("\n")), 1):
        if NATIVE_PROCESS.search(stripped):
            found.append(Finding("native-process-spawn", "medium", "runtime", path, number, raw.strip()[:160]))
        if NATIVE_NETWORK.search(stripped):
            found.append(Finding("native-network", "medium", "runtime", path, number, raw.strip()[:160]))
        if SENSITIVE_PATH.search(raw):
            found.append(Finding("credential-path", "high", "runtime", path, number, raw.strip()[:160]))
    return found


def scan_directory(directory: Path, pinned: set[str] | None = None) -> list[Finding]:
    """Findings for a corpus directory. With `pinned` (the manifest's paths), only those files are
    read: anything else is a build product or an edit, not what the pin says."""
    found: list[Finding] = []
    for file in sorted(directory.rglob("*")):
        if file.is_symlink() or not file.is_file():
            continue
        relative = file.relative_to(directory).as_posix()
        if pinned is not None and relative not in pinned:
            continue
        data = file.read_bytes()
        if is_forbidden_file(relative, data[:16]):
            found.append(Finding("binary-file", "high", "runtime", relative, 0, f"{len(data)} bytes"))
            continue
        if any(part in OFFICIAL_MODULE_NAMES for part in file.relative_to(directory).parts[:-1]):
            found.append(Finding("official-module-name", "low", "runtime", relative, 0, "directory named after an official module (not read, check provenance)"))
        suffix = file.suffix.lower()
        try:
            text = data.decode("utf-8", errors="replace")
        except Exception:  # pragma: no cover
            continue
        if suffix == ".jai":
            found.extend(scan_text(relative, text))
        elif suffix in NATIVE_SUFFIXES:
            found.extend(scan_native(relative, text))
    return found


def risk_of(findings: list[Finding]) -> str:
    return max((f.risk for f in findings), key=LEVEL.get, default="none")


def render(name: str, findings: list[Finding], verbose: bool = False, examples: int = 3) -> str:
    counts = {level: sum(f.risk == level for f in findings) for level in ("high", "medium", "low")}
    out = [f"{name}: {risk_of(findings).upper()} (high {counts['high']}, medium {counts['medium']}, low {counts['low']})"]
    for level in ("high", "medium", "low"):
        by_rule: dict[tuple[str, str], list[Finding]] = {}
        for f in findings:
            if f.risk == level:
                by_rule.setdefault((f.rule, f.context), []).append(f)
        for (rule, context), group in sorted(by_rule.items()):
            shown = group if verbose or level == "high" else group[:examples]
            out.append(f"  [{level}] {rule} ({context}) x{len(group)}")
            for f in shown:
                out.append(f"      {f.path}:{f.line}: {f.text}")
            if len(shown) < len(group):
                out.append(f"      ... {len(group) - len(shown)} more")
    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("repositories", nargs="*", help="directory names under the corpus (owner--repo) or owner/repo")
    parser.add_argument("--root", type=Path, default=None, help="directory holding the repositories (default corpus/upstream of the main checkout)")
    parser.add_argument("--json", type=Path, help="write findings as JSON")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()
    root = args.root
    if root is None:
        from repo_output import main_checkout

        root = main_checkout() / "corpus/upstream"
    names = [r.replace("/", "--", 1) for r in args.repositories] or sorted(
        p.name for p in root.iterdir() if p.is_dir() and not p.is_symlink() and not p.name.startswith(("_", "."))
    )
    manifest = ROOT / "corpus/upstreams.json"
    pins = {}
    if manifest.exists():
        for project in json.loads(manifest.read_text())["projects"]:
            # The pinned paths come from the Git cache the fetch left; without it, read every file.
            cache = cache_dir(project["repository"])
            if cache.exists() and project.get("file_count"):
                blobs = pinned_blobs(cache, project["repository"], project["revision"])
                pins[project["repository"].replace("/", "--", 1)] = set(blobs)
    report = {}
    worst = "none"
    for name in names:
        findings = scan_directory(root / name, pins.get(name))
        report[name] = {"risk": risk_of(findings), "findings": [asdict(f) for f in findings]}
        worst = max(worst, risk_of(findings), key=LEVEL.get)
        print(render(name, findings, args.verbose))
    if args.json:
        args.json.write_text(json.dumps(report, indent=1) + "\n")
    return 1 if worst == "high" else 0


if __name__ == "__main__":
    sys.exit(main())
