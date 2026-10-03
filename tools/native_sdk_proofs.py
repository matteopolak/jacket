#!/usr/bin/env python3
"""Reviewed source-built SDK ABI witnesses; saved evidence cannot authorize links."""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys

import native_dependencies as native
from native_sdk_oracles import FOCUS, IMGUI


@dataclass(frozen=True)
class SourcePin:
    name: str
    repository: str
    revision: str
    files: tuple[native.Fingerprint, ...]


@dataclass(frozen=True)
class RecordAbi:
    name: str
    size: int
    alignment: int
    offsets: tuple[int, ...]


@dataclass(frozen=True)
class ClassStorage:
    base_size: int
    derived_size: int
    context_offset: int


@dataclass(frozen=True)
class MeasuredAbi:
    records: tuple[RecordAbi, ...]
    draw_vertices: int | None = None
    draw_indices: int | None = None
    class_storage: ClassStorage | None = None
    selectors: tuple[str, ...] = ()


@dataclass(frozen=True)
class SdkWitness:
    source: SourcePin
    target: str
    compiler: native.Fingerprint
    archiver: native.Fingerprint
    symbol_tool: native.Fingerprint
    artifact: native.Fingerprint
    oracle_executable: native.Fingerprint
    transitive_inputs: tuple[native.Fingerprint, ...]
    flags: tuple[str, ...]
    required_symbols: tuple[str, ...]
    measured_abi: MeasuredAbi
    oracle_sha256: str
    link_authority: bool = False
    full_project_acceptance: bool = False


SOURCE_MANIFEST_SHA256 = 'bb8a777e07d04a556c0049ee276bc7cc68ff162ffc04b69db490bb9c90e29975'

REVISIONS = {
    'imgui-1.90.4-docking': ('ocornut/imgui','c6aa051629753f0ef0d26bf775a8b6a92aa213b2'),
    'focus-lightweight-view': ('focus-editor/focus','c6b3ead7d4174527d0138e8a31f7c3c5663badec'),
}
FILES = {
    'imgui-1.90.4-docking': ('imgui.cpp','imgui_draw.cpp','imgui_tables.cpp','imgui_widgets.cpp','imgui_demo.cpp',
                 'imgui.h','imgui_internal.h','imconfig.h','imstb_rectpack.h','imstb_textedit.h','imstb_truetype.h','LICENSE.txt'),
    'focus-lightweight-view': ('LightweightRenderingView.m','LightweightRenderingView.h','LICENSE'),
}
IMGUI_SYMBOLS = ('_ZN5ImGui13CreateContextEP11ImFontAtlas', '_ZN5ImGui14DestroyContextEP12ImGuiContext',
    '_ZN5ImGui10GetVersionEv', '_ZN5ImGui8NewFrameEv', '_ZN5ImGui6RenderEv',
    '_ZN5ImGui11GetDrawDataEv', '_ZN5ImGui21DockSpaceOverViewportEPK13ImGuiViewportiPK16ImGuiWindowClass',
    '_ZN10ImDrawList13AddRectFilledERK6ImVec2S2_jfi')
FOCUS_SYMBOLS = ('OBJC_CLASS_$_LightweightOpenGLView','OBJC_CLASS_$_LightweightRenderingView')
INSTALLED_HEADERS = tuple(map(Path,('/usr/include','/usr/lib','/Library/Developer','/Applications/Xcode.app','/opt/homebrew','/usr/local/Cellar')))


def measured_abi(name: str, output: str) -> MeasuredAbi:
    """Decode only the complete, bounded ABI protocol of a reviewed oracle."""
    expected = {
        'imgui-1.90.4-docking': {
            'ImVec2': (8, 4, 0, 4),
            'ImDrawVert': (20, 4, 0, 8, 16),
            'ImVectorPointer': (16, 8, 0, 4, 8),
            'ImGuiIO': (14616, 8),
            'ImGuiStyle': (1132, 4),
        },
        'focus-lightweight-view': {'NSRect': (32, 8, 0, 16)},
    }
    if name not in expected:
        raise ValueError('unreviewed SDK ABI protocol')
    rows = {}
    for line in output.splitlines():
        columns = line.split()
        if not columns or columns[0] in rows:
            raise ValueError('empty or duplicate SDK ABI row')
        rows[columns[0]] = columns[1:]
    extras = {'cpu_draw_counts'} if name.startswith('imgui') else {'class_storage', 'selectors'}
    if set(rows) != set(expected[name]) | extras:
        raise ValueError('SDK ABI protocol has missing or unknown rows')
    records = []
    for record, layout in expected[name].items():
        values = rows[record]
        if any(not re.fullmatch(r'[0-9]{1,9}', value) for value in values):
            raise ValueError('SDK ABI layout must contain bounded unsigned integers')
        decoded = tuple(map(int, values))
        if decoded != layout:
            raise ValueError(f'rebuilt SDK ABI differs from reviewed layout: {record}')
        records.append(RecordAbi(record, decoded[0], decoded[1], decoded[2:]))
    if name.startswith('imgui'):
        values = rows['cpu_draw_counts']
        if len(values) != 2 or any(not re.fullmatch(r'[0-9]{1,9}', value) for value in values):
            raise ValueError('invalid SDK CPU draw counts')
        vertices, indices = map(int, values)
        if vertices < 4 or indices < 6:
            raise ValueError('SDK CPU draw oracle produced no rectangle')
        return MeasuredAbi(tuple(records), vertices, indices)
    values = rows['class_storage']
    if len(values) != 3 or any(not re.fullmatch(r'[0-9]{1,9}', value) for value in values):
        raise ValueError('invalid Objective-C class storage')
    base, derived, offset = map(int, values)
    if base <= 0 or offset < base or offset % 8 or derived < offset + 8:
        raise ValueError('Objective-C context ivar is outside derived class storage')
    selectors = tuple(rows['selectors'])
    if selectors != ('object-getter', 'object-setter', 'void-swap'):
        raise ValueError('Objective-C selector ABI differs from reviewed contract')
    return MeasuredAbi(tuple(records), class_storage=ClassStorage(base, derived, offset), selectors=selectors)


def source_pin(name: str, source: Path, root: Path = native.ROOT) -> SourcePin:
    if name not in REVISIONS:
        raise ValueError('unreviewed SDK source recipe')
    source = native.outside_inputs(source,root)
    raw=(root/'corpus/native-sdk-sources.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!=SOURCE_MANIFEST_SHA256:
        raise ValueError('reviewed SDK source manifest fingerprint changed')
    manifest=json.loads(raw)
    if manifest.get('format')!=1 or manifest.get('kind')!='reviewed-source-sdk-recipes':
        raise ValueError('unsupported SDK source manifest')
    selected=[row for row in manifest['recipes'] if row['name']==name]
    if len(selected)!=1:
        raise ValueError('SDK source recipe must be unique')
    row=selected[0]
    if (row['repository'],row['revision'])!=REVISIONS[name]:
        raise ValueError('SDK source immutable revision changed')
    expected={item['name']:item['sha256'] for item in row['files']}
    if set(expected)!=set(FILES[name]) or len(expected)!=len(row['files']):
        raise ValueError('SDK source file set changed')
    actual=tuple(native.fingerprint(source/file,root) for file in FILES[name])
    if any(item.sha256!=expected[Path(item.path).name] for item in actual):
        raise ValueError('SDK source differs from reviewed immutable fingerprints')
    return SourcePin(name,*REVISIONS[name],actual)


def headers(paths: tuple[Path, ...], pin: SourcePin, generated: Path,
            root: Path) -> tuple[native.Fingerprint, ...]:
    reviewed={item.path:item.sha256 for item in pin.files}
    result=[]
    for path in sorted({path.resolve() for path in paths}):
        if path==generated:
            continue
        item=native.fingerprint(path,root)
        if item.path in reviewed:
            if item.sha256!=reviewed[item.path]:
                raise ValueError('reviewed SDK source changed during compilation')
        elif not any(path.is_relative_to(base) for base in INSTALLED_HEADERS):
            raise ValueError(f'unreviewed transitive SDK source: {path}')
        result.append(item)
    return tuple(result)


def symbol_names(text: str, target: str) -> frozenset[str]:
    symbols=set()
    for line in text.splitlines():
        columns=line.split()
        if len(columns)>=3 and re.fullmatch('[A-Za-z]',columns[-2]):
            name=columns[-1]
            if 'apple' in target and name.startswith('_'):
                name=name[1:]
            symbols.add(name)
    return frozenset(symbols)


def build(name: str, source: Path, target: str, output: Path,
          compiler: Path, archiver: Path, nm: Path, root: Path = native.ROOT) -> SdkWitness:
    pin=source_pin(name,source,root)
    if not re.fullmatch(r'(?:arm64|aarch64|x86_64)-(?:apple-macosx[0-9.]+|unknown-linux-gnu)',target):
        raise ValueError('SDK witnesses require explicit macOS/Linux target triples')
    if name=='focus-lightweight-view' and 'apple' not in target:
        raise ValueError('Focus Objective-C helper witness requires macOS')
    architecture=target.split('-')[0]
    actual=platform.machine().lower()
    normalize=lambda value: 'arm64' if value in {'arm64','aarch64'} else value
    if (normalize(architecture)!=normalize(actual)
            or ('apple' in target)!=(sys.platform=='darwin')
            or ('linux' in target)!=(sys.platform.startswith('linux'))):
        raise ValueError('native SDK execution witness requires the actual host architecture and OS')
    source=native.outside_inputs(source,root)
    output=native.outside_inputs(output,root)
    if output.exists() or not output.is_relative_to((root/'artifacts/native-dependencies').resolve()):
        raise ValueError('SDK output must be fresh beneath the managed rebuild root')
    cpp=native.installed_tool(compiler,root)
    ar=native.installed_tool(archiver,root)
    symbol_tool=native.installed_tool(nm,root)
    output.mkdir(parents=True)
    flags=('-std=c++17','-O2','-fPIC')
    objects=[]; dependencies=[]; commands=[]
    def run(command: list[str], timeout: int = 300):
        commands.append(command)
        with (output/'build.log').open('a') as log:
            log.write(json.dumps(command)+'\n'); log.flush()
            result=subprocess.run(command,env=native.clean_environment(),cwd=output,
                                  stdout=log,stderr=subprocess.STDOUT,timeout=timeout)
        if result.returncode:
            raise ValueError(f'fresh SDK command failed ({result.returncode}): '+(output/'build.log').read_text()[-5000:])
    for file in FILES[name]:
        if not file.endswith(('.cpp','.m')):
            continue
        obj=output/(Path(file).stem+'.o'); dep=obj.with_suffix('.d')
        command=[cpp.path,f'--target={target}',*flags,'-I',str(source),'-MD','-MF',str(dep)]
        if file.endswith('.m'):
            command+=['-x','objective-c++','-fno-objc-arc']
        command+=['-c',str(source/file),'-o',str(obj)]
        run(command); objects.append(obj); dependencies.extend(native.dependency_paths(dep))
    transitive=headers(tuple(dependencies),pin,output/'never-an-input',root)
    archive=output/'reviewed-sdk.a'
    run([ar.path,'rcsD',str(archive),*map(str,objects)])
    # Metadata inspection touches only this invocation's new archive.
    symbol_command=[symbol_tool.path,'--defined-only','--extern-only',str(archive)]
    commands.append(symbol_command)
    symbols=subprocess.run(symbol_command,
        env=native.clean_environment(),cwd=output,capture_output=True,text=True,timeout=30)
    if symbols.returncode:
        raise ValueError('fresh SDK symbol inspection failed')
    required=IMGUI_SYMBOLS if name.startswith('imgui') else FOCUS_SYMBOLS
    missing=set(required)-symbol_names(symbols.stdout,target)
    if missing:
        raise ValueError('rebuilt SDK is missing selected corpus symbols: '+repr(sorted(missing)))
    oracle=output/'oracle.mm'
    oracle.write_text(IMGUI if name.startswith('imgui') else FOCUS)
    dep=output/'oracle.d'; executable=output/'oracle'
    command=[cpp.path,f'--target={target}',*flags,'-I',str(source),'-MD','-MF',str(dep)]
    if name.startswith('imgui'):
        command+=['-x','c++',str(oracle),'-x','none']
    else:
        command+=['-fno-objc-arc',str(oracle),'-framework','AppKit','-framework','QuartzCore']
    command+=[str(archive),'-o',str(executable)]
    run(command)
    # Verify actual oracle inputs before the freshly built SDK can execute.
    oracle_headers=headers(native.dependency_paths(dep),pin,oracle,root)
    commands.append([str(executable)])
    measured=subprocess.run([str(executable)],env=native.clean_environment(),cwd=output,
                            capture_output=True,text=True,timeout=30)
    if measured.returncode:
        raise ValueError(f'fresh SDK ABI/runtime oracle failed ({measured.returncode}): '+measured.stderr[-3000:])
    abi=measured_abi(name,measured.stdout)
    witness=SdkWitness(pin,target,cpp,ar,symbol_tool,native.fingerprint(archive,root),
        native.fingerprint(executable,root),
        tuple(sorted(set(transitive+oracle_headers),key=lambda item:item.path)),flags,required,
        abi,native.sha256(oracle))
    (output/'receipt.json').write_text(json.dumps(asdict(witness),indent=2)+'\n')
    (output/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
    return witness


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('recipe',choices=tuple(REVISIONS))
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--target',required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--compiler',type=Path,default=Path('/usr/bin/clang++'))
    parser.add_argument('--archiver',type=Path,default=Path('/opt/homebrew/opt/llvm/bin/llvm-ar'))
    parser.add_argument('--nm',type=Path,default=Path('/opt/homebrew/opt/llvm/bin/llvm-nm'))
    args=parser.parse_args()
    witness=build(args.recipe,args.source,args.target,args.output,args.compiler,args.archiver,args.nm)
    print(json.dumps({'recipe':witness.source.name,'target':witness.target,'abi':asdict(witness.measured_abi),
                      'link_authority':False,'full_project_acceptance':False},indent=2))


if __name__=='__main__':
    main()
