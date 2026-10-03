"""Typed, read-only SDK configuration evidence; paths never authorize linking."""
from __future__ import annotations

from dataclasses import asdict, dataclass
import json
from pathlib import Path
import platform
import sys


@dataclass(frozen=True)
class SdkRoot:
    name: str
    path: Path

    @classmethod
    def parse(cls, value: str) -> 'SdkRoot':
        name, separator, path = value.partition('=')
        if not separator or name not in {'vulkan', 'sdl2', 'sdl3', 'glfw', 'slang', 'macos-sdk'}:
            raise ValueError('SDK configuration must be vulkan/sdl2/sdl3/glfw/slang/macos-sdk=absolute-root')
        candidate = Path(path)
        if not candidate.is_absolute():
            raise ValueError('SDK roots must be explicit absolute paths')
        return cls(name, candidate)


@dataclass(frozen=True)
class PathEvidence:
    name: str
    path: str
    exists: bool
    resolved_path: str | None
    role: str


def inventory(configured: tuple[SdkRoot, ...], root: Path) -> dict:
    # Import only on explicit inventory invocation, not native proof/rebuild paths.
    from native_dependencies import outside_inputs
    roots: dict[str, Path] = {}
    for item in configured:
        if item.name in roots:
            raise ValueError(f'duplicate SDK root configuration: {item.name}')
        roots[item.name] = outside_inputs(item.path, root)
    default = Path('/opt/homebrew')
    candidates: list[tuple[str, Path, str]] = []
    for sdk, rows in {
        'vulkan': [('vulkan-header', 'include/vulkan/vulkan.h', 'header'),
                   ('vulkan-loader', 'lib/libvulkan.dylib' if sys.platform == 'darwin' else 'lib/libvulkan.so', 'library')],
        'sdl2': [('sdl2-header', 'include/SDL2/SDL.h', 'header'),
                 ('sdl2-library', 'lib/libSDL2.dylib' if sys.platform == 'darwin' else 'lib/libSDL2.so', 'library')],
        'sdl3': [('sdl3-header', 'include/SDL3/SDL.h', 'header')],
        'glfw': [('glfw-header', 'include/GLFW/glfw3.h', 'header')],
        'slang': [('slang-header', 'include/slang.h', 'header')],
    }.items():
        base = roots.get(sdk, default)
        candidates.extend((name, base / relative, role) for name, relative, role in rows)
    sdk = roots.get('macos-sdk', Path('/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk'))
    candidates.append(('macos-sdk', sdk, 'sdk'))
    for name in ('AppKit', 'Foundation', 'Carbon', 'QuartzCore', 'CoreGraphics', 'GameController', 'Metal'):
        candidates.append((f'framework-{name}', sdk / f'System/Library/Frameworks/{name}.framework', 'framework'))
    candidates.append(('vma-header', default / 'include/vk_mem_alloc.h', 'header'))
    icd_directories = [roots.get('vulkan', default) / 'share/vulkan/icd.d',
                       roots.get('vulkan', default) / 'etc/vulkan/icd.d']
    if 'vulkan' not in roots:
        icd_directories.append(Path('/usr/share/vulkan/icd.d'))
    icds = []
    for directory in icd_directories:
        directory = outside_inputs(directory, root)
        if not directory.is_dir():
            continue
        for path in sorted(directory.glob('*.json')):
            path = outside_inputs(path, root)
            try:
                data = json.loads(path.read_text())
                row = data['ICD']
                library = row['library_path']
                if not isinstance(library, str):
                    raise ValueError('ICD library_path must be text')
                library_path = Path(library)
                if not library_path.is_absolute() and library_path.parent != Path('.'):
                    library_path = path.parent / library_path
                if library_path.is_absolute():
                    library_path = outside_inputs(library_path, root)
                icds.append({'manifest': str(path), 'api_version': row.get('api_version'),
                             'library_path': str(library_path),
                             'library_path_exists': library_path.is_file() if library_path.is_absolute() else None,
                             'runtime_verified': False})
            except (KeyError, json.JSONDecodeError, OSError, ValueError) as error:
                icds.append({'manifest': str(path), 'error': str(error), 'runtime_verified': False})
    rows = []
    for name, path, role in candidates:
        path = outside_inputs(path, root)
        exists = path.exists()
        rows.append(asdict(PathEvidence(name, str(path), exists, str(path.resolve()) if exists else None, role)))
    available = {row['name']: row['exists'] for row in rows}
    return {
        'kind': 'read-only-sdk-path-and-manifest-inventory',
        'host': {'platform': sys.platform, 'architecture': platform.machine()},
        'configured_roots': [{'name': item.name, 'path': str(roots[item.name])} for item in configured],
        'candidates': rows, 'icds': icds,
        'project_boundaries': [
            {'project': 'focus-editor/focus', 'native_acceptance': False,
             'macos_framework_paths_present': all(available[f'framework-{name}'] for name in ('AppKit','Foundation','Carbon','QuartzCore','CoreGraphics')),
             'remaining': ['Resolve selected target/feature module graph', 'Review/rebuild any selected local GUI or Tracy helper', 'Objective-C/GUI/event-lifetime execution']},
            {'project': 'ostef/Vk-Engine', 'native_acceptance': False,
             'host_matches_vulkan_imgui_source_branches': sys.platform in {'linux', 'win32'},
             'remaining': ['Reviewed Vulkan loader/ICD/GPU execution', 'SDL2 ABI contract', 'Version-matched ImGui C++ symbols/layouts', 'Target C++ runtime/unwind configuration']},
            {'project': 'roeyb1/sgpu', 'native_acceptance': False,
             'vulkan_header_and_loader_paths_present': available['vulkan-header'] and available['vulkan-loader'],
             'remaining': ['Reviewed full GPU VMA API binding contract', 'Reviewed Vulkan loader/ICD/GPU execution', 'Selected local/debug VMA mode', 'Slang source build and C++ binding contract when enabled']},
        ],
        'limitations': ['Path/header/ICD configuration evidence only; no library opened, GPU queried or native SDK executed.',
                        'Configured SDK roots do not grant native link authority.',
                        'Missing conventional paths do not establish absence elsewhere.',
                        'VMA virtual-allocation ABI execution does not validate Vulkan GPU allocation or complete project dependencies.'],
    }
