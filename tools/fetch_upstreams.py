#!/usr/bin/env python3
"""Pin recent upstream Jai sources as inert compatibility input, never run them."""

from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
import subprocess
import urllib.parse

from repo_output import main_checkout, shared_output_dir
from toolkit.download import fetch as fetch_url
from toolkit.repo import ROOT

REPOSITORIES = (
    "focus-editor/focus",
    "Ivo-Balbaert/The_Way_to_Jai",
    "SogoCZE/Jails",
    "rluba/jaison",
    "withlang-dev/open-jai",
    "ostef/Vk-Engine",
    "roeyb1/sgpu",
    # Smoke-test set: the language:Jai repositories with 20+ stars pushed since 2026-05-05
    # (docs/tools/upstream-corpus.md explains how to refresh it). abhiramasonny/jaithon
    # matched too but is its own language that happens to use the .jai extension.
    "danieltan1517/chess-jai",
    "gabrielmfern/forbear",
    "rexim/rexim.github.io",
    "kooparse/ui_builder",
    "DavidColson/Photon",
    "kujukuju/KodaJai",
    "UnNabbo/no_api",
    "ahmedqarmout2/raylib-jai",
)
# Libraries the projects above import from git submodules: pinned like the
# projects but exempt from the recency cutoff.
DEPENDENCIES = (
    "SogoCZE/jai_parser",
    "ostef/Linalg",
    "ostef/Jolt-Jai",
    "ostef/JoltC",
    "wolfpld/tracy",
    "OrangeLightning219/jai_parser",
    # Modules KodaJai imports by name (no submodules or manifest: the newest commits).
    "kujukuju/JaiMath",
    "kujukuju/JaiGLFW",
    "kujukuju/KodaPhysics",
    "kujukuju/KodaSerializer",
    "kujukuju/JaiWasmGL",
    "kujukuju/JaiParallel",
    "kujukuju/JaiBoundingTree",
    "kujukuju/FixedStringJai",
    "kujukuju/ContiguousJsonJai",
    "kujukuju/BlockAllocatorJai",
    "kujukuju/lz4_static",
    "kujukuju/JaiSerializer",
    # Modules other pinned projects import by name.
    "WeirdBirds/Jay_Utils",
    "WeirdBirds/Jay_Core",
    "WeirdBirds/Jay_Logger",
    "jatinchowdhury18/clap-jai",
    "jatinchowdhury18/clap-jai-plugin-base",
)
# Metaprogramming libraries that exercise the Compiler module's node API (typed
# compiler_get_nodes, compiler_get_code, TYPECHECKED bodies). Few change often, so they
# are exempt from the recency cutoff too.
LIBRARIES = (
    "sjorsdonkers/match-jai",
    "sjorsdonkers/yield-jai",
    "GufNZ/JaiModules-AST_Utils",
    "PixelRifts/Jai-Shader-Transpiler",
    "Stuart-Mouse/jai-utils",
    "n00bmind/unotest",
    # rluba's library family (jaison is a project above); several import each other.
    "rluba/jai-tracy",
    "rluba/jai-redis",
    "rluba/uniform",
    "rluba/cluster",
    "rluba/jai-csv",
    "rluba/jai-postgres",
    "rluba/stubborn",
    "rluba/hyperserve",
    "rluba/wait_group",
    "rluba/jai-date",
    # Libraries with their own test suites or self-checking examples: sweep cases check their
    # results (docs/tools/upstream-corpus.md, "Recorded outputs").
    "sjorsdonkers/toml-jai",
    "OrangeLightning219/jai-format",
    "segcore/jai-protobuf",
    "n00bmind/reflector",
    "smari/jai-xml",
)
# Every language:Jai repository with at least STAR_THRESHOLD stars as of 2026-10-20, regardless of
# when it last changed (`gh search repos --language=Jai --stars=">=5"`), plus the code-search finds
# (`extension:jai` queries) whose language GitHub misreports or leaves empty. Exempt from the
# recency cutoff like LIBRARIES, but unreviewed: run tools/corpus/scan_upstream.py and read its
# high-risk findings before a repository is listed here, only text sources are taken (see
# `is_forbidden_file`), vendored copies of official Jai modules are left out
# (`vendored_official_module`), and the cases of these repositories carry `"review": "pending"`
# in tools/upstream-cases.json (docs/tools/upstream-corpus.md, "Review gate").
STAR_THRESHOLD = 5
STAR_REPOSITORIES = (
    "valignatev/hitboxer",
    "rezich/Enumerated_Array",
    "charlesastaylor/jai-raddebugger",
    "onelivesleft/jai-string",
    "nick-celestin-zizic/joogle",
    "isaplane/sdl3-jai",
    "seg4lt/zed-jai",
    "asmvik/spaced",
    "mimhufford/AdventOfCode",
    "666rayen999/rjyv",
    "ostef/OhMyGUI",
    "roeyb1/jai-wavefront",
    "Breush/lava-jai",
    "dbechrd/jai-yojimbo",
    "filippocrocchini/Glyph_Cache",
    "isomorphic-tech/fuzzy-match-jai",
    "yasinkaraaslan/Gaco",
    "ostef/Jai-Win32",
    "bugulo/jai-glfw",
    "kujukuju/JaiSteamworks",
    "wrapperup/mew",
    "Sl3dge78/JaiBoy",
    "WeirdBirds/Jay_Math",
    "666rayen999/hash_map",
    "charlesastaylor/Performance_Monitoring_Counters",
    "smari/jai-crypto",
    "dbechrd/jai-simpler",
    "de-senk/Physics_AVBD_2D",
    "superjustin5000/tools_jai",
    "isomorphic-tech/jai-sodium",
    "Lunatoid/Rematch",
    "SamuelDeboni/Microui-Jai",
    "shiMusa/MathExtensions",
    "drshapeless/vulkan-jai-binding",
    "WeirdBirds/Jay_ECS",
    "isomorphic-tech/jai-sqlite",
    "666rayen999/qoa-jai",
    "kujukuju/KodaPathFinding",
    "tsoding/jaibreak",
    "valignatev/jai-on-linux",
    "SogoCZE/jai_wgpu_native",
    "danieltan1517/orange-xiangqi",
    "fda0/Treasure",
    "judah-caruso/base.jai",
    "rubenbala/uppies",
    "farzher/clipman",
    "qiekn/ootss-cmd",
    "TheGag96/raspberry-jai",
    "TraceMyers/vai",
    "MariuzM/jai-lsp-scratch",
    "Breush/sloubi-chess",
    "yanchith/editor",
    "QuattroMusic/6502-Simulator",
    # Found with GitHub code search (`extension:jai`) because GitHub labels them as another language
    # or none.
    "LuisThiamNye/ButteryTaskbar2",
    "daafu/sokoban",
    "dlandahl/theos-2",
    "randyprime/resources",
    "asbott/One-Std",
    "tsoding/jai-wasm",
    "farzher/Jai-HTTP-Server",
    "okmatija/Prizm",
    "tsoding/voronoi-browser",
    "nafonso/netrix",
    "acoto87/war1-jai",
    "rluba/jai-ctags",
    "666rayen999/x-math",
    "isomorphic-tech/jai-sdl3",
    "Chowdhury-DSP/wdf_compiler",
    "tsoding/rope-jai",
    "tsoding/tetris-jai",
    "valignatev/jai-control-flow",
    "rluba/smash",
    "rexim/aoc-2023",
    "onelivesleft/the-language",
    "tsoding/ditch",
    "nick-celestin-zizic/clay-jai",
    "onelivesleft/jai-steam",
    "kooparse/gltf_parser",
    "colinbellino/sokol-jai",
    "rexim/aoc-2022",
    "onelivesleft/jai-cookbook",
    "farzher/Bunnymark-Jai-D3D11",
    "smari/jai-simplehttp",
    "isomorphic-tech/jai-imgui",
    "rytc/enet-jai",
    "rezich/SQLite",
    "SogoCZE/jai_wasm",
    "PAYLHORSE-Software/Jodot",
    "tsoding/infinite-pong-jai-wasm64",
    "judah-caruso/jai-sokol",
    "charlesastaylor/jai-ffmpeg",
    "Cakez77/terraria_clone",
    "y3konline/bonk",
    "rluba/twitch_irc",
    "napina/jai_cgltf",
    "sivertjoe/Advent-of-Code",
    "rluba/bureaucrat",
    "kujukuju/PlaydateJai",
    "kevinw/jai-shooter",
    "forrestthewoods/aoc2022",
    "contextfreecode/jai-comparisons",
    "colinbellino/jai-code-hot-reloading-example",
    "FinochioM/Makis_ECS",
    "y3konline/jai-recastnavigation",
    "ryanneph/Jai-Code-Interop",
    "mathaou/jai-simpd",
    "kujukuju/JaiBullet",
    "dbechrd/jai-assertive",
    "acoto87/jai-modules",
    "LainLayer/XChatger",
    "CyanMARgh/jaigl",
    "Breush/lava-jai-bindings",
    "wijiler/Quartzz",
    "smari/Cavy",
    "kujukuju/JaiPhysX",
    "filippocrocchini/Harfbuzz",
    "666rayen999/recs",
    "rytc/slang-jai",
    "rluba/jai-unicode",
    "mimhufford/jaidoku",
    "kooparse/miniaudio-jai",
    "kevinw/jaitro",
    "jatinchowdhury18/clap-jai-example-plugin",
    "colinbellino/jai-sokol-wasm-playground",
    "baileysostek/Jai-Vulkan",
    "asmvik/assimp-jai",
    "arjenwitteveen/jai-midi",
    "Manquia/gns-jai",
    "CyanMARgh/raylib-jai-universal",
)
# Star-qualified repositories that are deliberately not pinned, with the reason.
EXCLUDED_REPOSITORIES = {
    "abhiramasonny/jaithon": "its own language that happens to use the .jai extension",
    "matteopolak/jacket": "this project",
    "AlDanial/cloc": "its one .jai file is a test input for a line counter",
    "Chowdhury-DSP/BYOD": "an audio plugin in C++ with a small incidental Jai effect",
    "HGitMaster/geotools-osgi": "a Java project whose .jai files are unrelated registry files",
    "RobinWragg/JaiTools": "a syntax-highlighting test file",
    "Sharir/jai": "2016 pre-release syntax experiments, not real Jai",
    "ThePrimeagen/ts-rust-zig-deez": "one of many language implementations in a comparison repository",
    "Thekla/thekla_atlas": "a C++ library with one incidental Jai binding file",
    "Zxynine/EvenBetterComments": "an editor extension with one sample file",
    "cirosantilli/china-dictatroship-7": "spam repository with a hello-world sample",
    "rytc/vulkan-jai": "a vendored copy of an official module",
    "cmuratori/computer_enhance": "a course repository whose only Jai is a generated bindings example",
    "constantitus/tree-sitter-jai": "a grammar repository whose only .jai file is a grammar test",
    "contextfreecode/boolerr": "a Haskell project with one incidental Jai file",
    "contextfreecode/procfun": "a Zig project with one incidental Jai file",
    "github-linguist/linguist": "language-detection samples, not a Jai project",
    "mRFWq7LwNPZjaVv5v6eo/cihna-dictattorshrip-8": "spam repository with a hello-world sample",
    "mmbase/mmbase": "a Java project whose .jai file is a licence text",
    "noqcks/generated": "file-detection samples, not a Jai project",
    "pixeldroid/jailang": "a documentation site with code snippets, written for pre-release Jai",
    "pmouraguedes/neodarcula.nvim": "an editor theme with one sample file",
    "seanpm2001/Git-Templates": "licence templates saved with a .jai extension",
    "subamanis/mezura": "a Rust tool whose only .jai file is a test fixture",
    "travisdoor/tine": "a compiler whose only .jai file is a syntax test",
    "tsoding/coroutines": "a C library whose one .jai file is an example",
    "willdoescode/hello-world": "one hello-world file in a multi-language repository",
}
# Dependencies pinned to the consumer's submodule commit (else the newest commit).
SUBMODULE_REVISIONS = {
    "ostef/Linalg": "5f60c11f057787a804ce9582c5daeccc0e8de89a",
    "ostef/Jolt-Jai": "56d1cd47b92d6c08ecdab5b9057addce35dab41a",
    # Jolt-Jai's Source/JoltC submodule: the C wrapper around Jolt Physics that libJoltC is built from.
    "ostef/JoltC": "d395f4138e1d29dfd46884f6ba00ff865248af08",
    # rluba/jai-tracy's `tracy` submodule (Tracy v0.11.1): libtracy is built from its client sources.
    "wolfpld/tracy": "30997d5ca6bb632cc10807a1da8a6d3de0aeeb3c",
    # jai-format's `modules/jai_parser` submodule (a fork of SogoCZE/jai_parser).
    "OrangeLightning219/jai_parser": "7745b7960f3af8f090e089fceb89a28ead151c40",
}
# Repositories of which only these path prefixes are taken (plus licenses and READMEs): Tracy's
# profiler GUI, server and bundled libraries are not needed to build its client library.
SOURCE_PREFIXES = {
    "wolfpld/tracy": ("public/",),
}
# Data files a project reads at compile time (`#run read_entire_file`, fonts...),
# by path prefix.
RESOURCE_PREFIXES = {
    "focus-editor/focus": ("config/", "fonts/", "images/", "themes/"),
    # sgpu examples compile their Slang shaders at run time and one loads a sample texture.
    "roeyb1/sgpu": ("examples/shaders/", "examples/sample.png"),
    # JoltC builds with CMake (tools/native/build_vk_engine_libs.py); Jolt Physics itself is cloned there.
    "ostef/JoltC": ("CMakeLists.txt", "Examples/"),
    # chess-jai reads its NNUE network at compile time and its fonts, images and sounds at run time.
    "danieltan1517/chess-jai": ("resources/",),
    "kooparse/ui_builder": ("demo/assets/",),
    "gabrielmfern/forbear": ("apps/Inter.ttf",),
    # 30.15_global_data embeds this image with #run add_global_data and prints its size.
    "Ivo-Balbaert/The_Way_to_Jai": ("examples/30/pixel.png",),
    # jai-xml's test.jai parses every file under test_data/ (the committed subset of its suite).
    "smari/jai-xml": ("test_data/",),
    # jai-protobuf's tests generate Jai code from these schemas.
    "segcore/jai-protobuf": ("examples/protos/",),
}
# Per-repository download cap where a needed data file is bigger than the default 8 MiB
# (chess-jai's network is 21 MB).
MAX_FILE_BYTES = {"danieltan1517/chess-jai": 32 * 1024 * 1024}
# Submodule mount points: (consumer directory link, target relative to corpus/upstream).
MODULE_LINKS = (
    ("SogoCZE--Jails/modules/jaison", "rluba--jaison"),
    ("SogoCZE--Jails/modules/unicode_utils", "rluba--jaison/unicode_utils"),
    ("SogoCZE--Jails/modules/jai_parser", "SogoCZE--jai_parser"),
    (
        "OrangeLightning219--jai-format/modules/jai_parser",
        "OrangeLightning219--jai_parser",
    ),
    ("ostef--Vk-Engine/Modules/Linalg", "ostef--Linalg"),
    ("ostef--Vk-Engine/Modules/JoltPhysics", "ostef--Jolt-Jai"),
    ("ostef--Jolt-Jai/Source/JoltC", "ostef--JoltC"),
    # Libraries that are imported as modules by name (`#import "AST_Utils"`).
    ("_modules/AST_Utils", "GufNZ--JaiModules-AST_Utils"),
    ("rluba--jai-tracy/tracy", "wolfpld--tracy"),
    # rluba's modules import each other by these names (`#import "date"`, `"wait_group"`...).
    ("_modules/uniform", "rluba--uniform"),
    ("_modules/wait_group", "rluba--wait_group"),
    ("_modules/cluster", "rluba--cluster"),
    ("_modules/date", "rluba--jai-date"),
    ("_modules/stubborn", "rluba--stubborn"),
    ("_modules/tracy", "rluba--jai-tracy"),
    ("_modules/hyperserve", "rluba--hyperserve"),
    # uniform's build file loads its test runner from its own `modules/stubborn`.
    ("rluba--uniform/modules/stubborn", "rluba--stubborn"),
    # KodaJai imports its author's modules by name.
    ("_modules/KodaJai", "kujukuju--KodaJai"),
    ("_modules/JaiMath", "kujukuju--JaiMath"),
    ("_modules/JaiGLFW", "kujukuju--JaiGLFW"),
    ("_modules/KodaPhysics", "kujukuju--KodaPhysics"),
    ("_modules/KodaSerializer", "kujukuju--KodaSerializer"),
    ("_modules/JaiWasmGL", "kujukuju--JaiWasmGL"),
    ("_modules/JaiParallel", "kujukuju--JaiParallel"),
    ("_modules/JaiBoundingTree", "kujukuju--JaiBoundingTree"),
    ("_modules/FixedStringJai", "kujukuju--FixedStringJai"),
    ("_modules/ContiguousJsonJai", "kujukuju--ContiguousJsonJai"),
    ("_modules/BlockAllocatorJai", "kujukuju--BlockAllocatorJai"),
    ("_modules/lz4_static", "kujukuju--lz4_static"),
    # Star-qualified repositories whose sources import their own module name (`#import "OhMyGUI"`).
    ("_modules/OhMyGUI", "ostef--OhMyGUI"),
    ("_modules/Yojimbo", "dbechrd--jai-yojimbo"),
    ("_modules/Jay_ECS", "WeirdBirds--Jay_ECS"),
    ("_modules/Jay_Math", "WeirdBirds--Jay_Math"),
    ("_modules/Jay_Utils", "WeirdBirds--Jay_Utils"),
    ("_modules/Jay_Core", "WeirdBirds--Jay_Core"),
    ("_modules/Jay_Logger", "WeirdBirds--Jay_Logger"),
    ("_modules/JaiSerializer", "kujukuju--JaiSerializer"),
    ("_modules/clap-jai", "jatinchowdhury18--clap-jai"),
    ("_modules/clap-jai-plugin-base", "jatinchowdhury18--clap-jai-plugin-base"),
    ("_modules/Win32", "ostef--Jai-Win32"),
    ("_modules/SDL3", "isaplane--sdl3-jai"),
)

DATA = main_checkout()
# Repositories whose newest Jai change may predate --since.
RECENCY_EXEMPT = DEPENDENCIES + LIBRARIES + STAR_REPOSITORIES


def git(cache: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", "--git-dir", str(cache), *args], text=True
    ).strip()


def safe_path(path: str) -> PurePosixPath:
    p = PurePosixPath(path)
    if p.is_absolute() or ".." in p.parts or not p.parts:
        raise ValueError(f"unsafe upstream path: {path}")
    return p


def download(repo: str, revision: str, path: str) -> bytes:
    safe_path(path)
    url = f"https://raw.githubusercontent.com/{repo}/{revision}/{urllib.parse.quote(path)}"
    limit = MAX_FILE_BYTES.get(repo, 8 * 1024 * 1024)
    try:
        return fetch_url(
            url,
            timeout=60,
            headers={"User-Agent": "jai-rust-compatibility-research"},
            max_bytes=limit,
        )
    except ValueError:
        raise ValueError(f"oversize source: {repo}/{path}") from None


# C-family sources a project builds its own helper libraries from (never prebuilt binaries).
NATIVE_SOURCE_SUFFIXES = (".c", ".h", ".m", ".mm", ".cpp", ".cc", ".hpp", ".inl")
# Never pinned from any repository, whatever a RESOURCE_PREFIXES entry says: compiled code,
# object files, archives and installers. (Fonts, images and data files stay allowed by prefix.)
BINARY_SUFFIXES = (
    ".dll", ".so", ".dylib", ".exe", ".a", ".lib", ".o", ".obj", ".bin", ".pdb", ".sys", ".node",
    ".wasm", ".jar", ".class", ".apk", ".ipa", ".msi", ".dmg", ".pkg", ".deb", ".rpm", ".app",
    ".zip", ".tar", ".gz", ".tgz", ".bz2", ".xz", ".zst", ".7z", ".rar", ".cab", ".iso", ".pyc",
)
# Leading bytes of executables, libraries and archives: refused even under a harmless name.
BINARY_MAGIC = (
    b"MZ", b"\x7fELF", b"\xfe\xed\xfa\xce", b"\xfe\xed\xfa\xcf", b"\xce\xfa\xed\xfe",
    b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe", b"!<arch>", b"PK\x03\x04", b"\x1f\x8b",
    b"\x00asm", b"7z\xbc\xaf", b"Rar!",
)
# Names of the modules the official Jai distribution ships. A repository that vendors a copy keeps
# it out of the pin: the corpus must not contain official sources.
OFFICIAL_MODULE_NAMES = frozenset(
    """Basic Math String File File_Utilities Hash_Table Hash Random Thread Process Socket Simp GL Input
    Window_Creation Window_Type Compiler Reflection Sort Unicode Command_Line Bit_Operations Bit_Array
    Bucket_Array Pool Flat_Pool Atomics System Runtime_Support Default_Allocator Memory Print_Color
    Text_File_Handler Wav_File Sound_Player Debug Check Clipboard Curl Deep_Copy Gamepad GetRect
    Jai_Lexer Keymap Machine_X64 POSIX Windows Linux X11 SDL freetype stb_image stb_image_write
    stb_image_resize stb_vorbis Vulkan ImGui Metal Objective_C MacOS_Bundler Base64 Crc md5 xxHash
    meow_hash lz4 Zip_File_Directory Program_Print Iprof Treemap IntroSort RadixSort Relative_Pointers
    Remap_Context Soa SIMD Srgb Subtitles Tracy PCG Float16 Code_Visit Color_Interpolation
    Default_Metaprogram Minimal_Metaprogram Metaprogram_Plugins Bindings_Generator Generate_C_Header
    Project_Generator Windows_Registry Windows_Resources Windows_Utf8 Autorun BuildCpp Icon Ico_File
    Mail File_Watcher Debug_Info Overwriting_Allocator Unmapping_Allocator Performance_Report
    Print_Vars Rewrite Sloppy_Math Codex d3d11 d3d12 dxgi d3d_compiler dxc_compiler rpmalloc meshoptimizer
    nvtt pl_mpeg MojoShader Android iOS macOS""".split()
)


def is_forbidden_file(path: str, data: bytes | None = None) -> bool:
    """Binary artefacts: by suffix, and (once downloaded) by magic bytes."""
    name = PurePosixPath(path).name.lower()
    if name.endswith(BINARY_SUFFIXES) or re.search(r"\.so\.\d", name):
        return True
    return data is not None and data.startswith(BINARY_MAGIC)


# The same names compared without case: a vendored copy is often renamed to lower case
# (`modules/autorun.jai`, `curl/module.jai`).
OFFICIAL_MODULE_KEYS = frozenset(name.lower() for name in OFFICIAL_MODULE_NAMES)


def vendored_official_module(path: str, paths: list[str]) -> bool:
    """Whether `path` is, by its location alone, part of a copy of an official module. Only the
    names of files and directories are looked at, compared without case:

    - a directory named after an official module that holds its own `module.jai` (a vendored
      copy), and everything below it;
    - a directory named after one that sits directly inside a `modules` directory, with or
      without a `module.jai` (copies often keep the module's own file names instead);
    - the single-file form of one, directly inside a `modules` directory (`modules/Basic.jai`)."""
    parts = PurePosixPath(path).parts
    for i, part in enumerate(parts[:-1]):
        if part.lower() not in OFFICIAL_MODULE_KEYS:
            continue
        if "/".join(parts[: i + 1] + ("module.jai",)) in paths:
            return True
        if i > 0 and parts[i - 1].lower() == "modules":
            return True
    stem = PurePosixPath(path).stem
    return (
        len(parts) > 1
        and parts[-2].lower() == "modules"
        and stem.lower() in OFFICIAL_MODULE_KEYS
        and path.endswith(".jai")
    )


# Directories whose names match an official module (without case) but that are the repository's
# own work: bindings for third-party C libraries, or a project's own parser. Reviewed by their
# place in the repository (a sokol binding next to the other sokol bindings, a module inside an
# operating system project's own `modules` directory), never by comparing with official sources.
OWN_MODULE_PREFIXES = {
    "colinbellino/jai-sokol-wasm-playground": ("src/modules/sokol-jai/sokol/gl/",),
    "colinbellino/sokol-jai": ("sokol/gl/", "sokol/imgui/"),
    "dlandahl/theos-2": ("kernel/modules/debug_info/",),
    "okmatija/Prizm": ("modules/tracy/",),
}


def blob_id(data: bytes) -> str:
    """The Git object id of a file's content."""
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def tree_blobs(cache: Path, revision: str) -> dict[str, str]:
    """path -> Git object id of every file at `revision` (submodules excluded). Needs no file
    contents, so it works on the blobless cache."""
    blobs = {}
    for entry in git(cache, "ls-tree", "-r", "-z", revision).split("\0"):
        if entry:
            meta, path = entry.split("\t", 1)
            _mode, kind, oid = meta.split()
            if kind == "blob":
                blobs[path] = oid
    return blobs


def select_paths(repo: str, paths: list[str]) -> list[str]:
    """The files of a repository that are pinned: Jai and C-family sources, READMEs, licences and
    the repository's resource prefixes, never binaries or vendored official modules."""
    if repo in SOURCE_PREFIXES:
        paths = [p for p in paths if p.startswith(SOURCE_PREFIXES[repo]) or "/" not in p]
    every_path = set(paths)
    return sorted(
        p
        for p in paths
        if not is_forbidden_file(p)
        and not (
            repo in STAR_REPOSITORIES
            and vendored_official_module(p, every_path)
            and not p.startswith(OWN_MODULE_PREFIXES.get(repo, ()))
        )
        and (
            p.endswith(".jai")
            or p.endswith(NATIVE_SOURCE_SUFFIXES)
            or PurePosixPath(p).name.lower() in {"copying", "readme.md"}
            or PurePosixPath(p).name.lower().startswith("license")
            or (p.startswith(RESOURCE_PREFIXES.get(repo, ())) and not p.endswith(".psd"))
        )
    )


def pinned_blobs(cache: Path, repo: str, revision: str) -> dict[str, str]:
    """The pinned files of `repo` at `revision` with their Git object ids. The manifest records only
    the revision: the commit fixes every file's content, and this selection fixes which files."""
    blobs = tree_blobs(cache, revision)
    return {p: blobs[p] for p in select_paths(repo, list(blobs))}


def cache_dir(repo: str) -> Path:
    return shared_output_dir("cache", "upstream") / f"{repo.replace('/', '--')}.git"


def fetch(repo: str, since: datetime, pinned: dict | None = None) -> dict:
    key = repo.replace("/", "--")
    cache = cache_dir(repo)
    cache.parent.mkdir(parents=True, exist_ok=True)
    if not cache.exists():
        subprocess.run(
            [
                "git",
                "clone",
                "--bare",
                "--filter=blob:none",
                "--depth=128",
                f"https://github.com/{repo}.git",
                str(cache),
            ],
            check=True,
        )
    # Existing manifest pins win even on a machine with no cache.
    revision = (
        pinned["revision"]
        if pinned
        else SUBMODULE_REVISIONS.get(repo) or git(cache, "rev-parse", "HEAD")
    )
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError(f"invalid revision: {revision}")
    if subprocess.run(
        ["git", "--git-dir", str(cache), "cat-file", "-e", revision + "^{commit}"],
        capture_output=True,
    ).returncode:
        subprocess.run(
            [
                "git",
                "--git-dir",
                str(cache),
                "fetch",
                "--filter=blob:none",
                "--depth=128",
                "origin",
                revision,
            ],
            check=True,
        )
    stamp = git(cache, "log", "-1", "--format=%cI", revision, "--", "*.jai")
    if not stamp and repo in DEPENDENCIES + LIBRARIES:
        stamp = git(
            cache, "log", "-1", "--format=%cI", revision
        )  # native-only dependency (no .jai files)
    if not stamp:
        raise ValueError(f"no Jai source changes in 128 commits: {repo}")
    source_date = datetime.fromisoformat(stamp.replace("Z", "+00:00"))
    record = {
        "repository": repo,
        "revision": revision,
        "latest_jai_change": stamp,
        "source_of_truth": f"https://github.com/{repo}/tree/{revision}",
    }
    if source_date < since and repo not in RECENCY_EXEMPT:
        return record | {"selection": "stale-excluded", "file_count": 0}
    blobs = pinned_blobs(cache, repo, revision)
    destination = DATA / "corpus/upstream" / key

    def one(path: str) -> None:
        target = destination.joinpath(*safe_path(path).parts)
        if target.is_file() and blob_id(target.read_bytes()) == blobs[path]:
            return
        data = download(repo, revision, path)
        if is_forbidden_file(path, data):
            raise ValueError(f"binary content refused: {repo}/{path}")
        if blob_id(data) != blobs[path]:
            raise ValueError(f"download differs from the Git object (truncated?): {repo}/{path}")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    with ThreadPoolExecutor(max_workers=8) as pool:
        list(pool.map(one, sorted(blobs)))
    return record | {"selection": "recent-source", "file_count": len(blobs)}


# Windows: "A required privilege is not held by the client" (creating a symlink).
ERROR_PRIVILEGE_NOT_HELD = 1314


def link_folder(path: Path, target: Path) -> None:
    """Point `path` at the folder `target` with a relative symlink. Windows refuses symlinks without
    Developer Mode or elevation, so there it falls back to a directory junction (absolute, no privilege)."""
    if path.is_symlink():
        path.unlink()
    elif path.is_dir():
        os.rmdir(path)  # a junction from an earlier run; a real folder with files in it is not removed
    elif path.exists():
        path.unlink()
    try:
        path.symlink_to(os.path.relpath(target, path.parent), target_is_directory=True)
    except OSError as error:
        if getattr(error, "winerror", None) != ERROR_PRIVILEGE_NOT_HELD:
            raise
        import _winapi

        _winapi.CreateJunction(str(target.resolve()), str(path))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--since",
        default="2025-10-01",
        help="minimum Jai source modification date (UTC); STAR_REPOSITORIES, DEPENDENCIES and LIBRARIES are exempt",
    )
    parser.add_argument(
        "--only",
        nargs="+",
        metavar="OWNER/REPO",
        help="fetch just these repositories at their pinned revisions (`_modules` stands for "
        "the libraries linked there); the manifest is left alone",
    )
    args = parser.parse_args()
    since = datetime.fromisoformat(args.since).replace(tzinfo=timezone.utc)
    manifest = ROOT / "corpus/upstreams.json"
    old = json.loads(manifest.read_text()) if manifest.exists() else {"projects": []}
    pins = {p["repository"]: p for p in old["projects"]}
    projects = []
    wanted = None
    if args.only:
        wanted = set(args.only)
        if "_modules" in wanted:
            wanted |= {
                t.replace("--", "/", 1)
                for line, t in MODULE_LINKS
                if line.startswith("_modules/")
            }
        # Submodule mount points name their targets (Jails needs jaison and jai_parser).
        for link, target in MODULE_LINKS:
            if link.split("/")[0].replace("--", "/", 1) in wanted:
                wanted.add(target.split("/")[0].replace("--", "/", 1))
    for repo in REPOSITORIES + DEPENDENCIES + LIBRARIES + STAR_REPOSITORIES:
        if wanted is not None and repo not in wanted:
            continue
        record = fetch(repo, since, pins.get(repo))
        projects.append(record)
        print(
            f"{repo}: {record['selection']}, {record['file_count']} text files at {record['revision']}",
            flush=True,
        )
    upstream = DATA / "corpus/upstream"
    for link, target in MODULE_LINKS:
        path = upstream / link
        if wanted is not None and not (
            (upstream / target).exists()
            and (
                link.startswith("_modules/") or (upstream / link.split("/")[0]).exists()
            )
        ):
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        link_folder(path, upstream / target)
    if wanted is not None:
        return
    temporary = manifest.with_suffix(".json.tmp")
    temporary.write_text(
        json.dumps(
            {
                "format": 2,
                "minimum_source_date": since.isoformat(),
                "projects": projects,
            },
            indent=2,
        )
        + "\n"
    )
    temporary.replace(manifest)


if __name__ == "__main__":
    main()
