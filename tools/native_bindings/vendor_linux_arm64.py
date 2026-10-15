#!/usr/bin/env python3
"""Private Linux ARM64 kernel ABI from complete pinned official UAPI C headers.

The Bionic archive carries the official exported Linux UAPI header graph.
Only kernel structures/numbers are used, not Bionic userspace implementation.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import tarfile

from vendor_records import record_shape, assertions, run, digest

ROOT = Path(__file__).resolve().parents[2]

OUTPUT = ROOT/'stdlib/internal/bindings/linux-arm64-kernel/abi.jai'
INCLUDES = '''#include <asm/stat.h>
#include <asm/unistd.h>
#include <linux/fcntl.h>
#include <stddef.h>
enum BindingKernelConstants {
    SYS_FSTATAT = __NR_newfstatat,
    SYS_FSTAT = __NR_fstat,
    CURRENT_DIRECTORY = AT_FDCWD,
    NOFOLLOW = AT_SYMLINK_NOFOLLOW
};
'''


def folded_value(node):
    if node['kind']=='ConstantExpr' and 'value' in node:
        return int(node['value'])
    for child in node.get('inner',[]):
        value = folded_value(child)
        if value is not None:
            return value
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    parser.add_argument('--formatter',type=Path)
    parser.add_argument('--report',type=Path,default=ROOT/'.work/linux-arm64-kernel-header-check.json')
    args = parser.parse_args()
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['bionic']
    archive = ROOT/'.work/vendor/bionic.tar.gz'
    if digest(archive.read_bytes()) != pin['archive']['sha256']:
        raise SystemExit('Pinned native UAPI archive differs')
    vendor = ROOT/'.work/vendor'/pin['archive']['root']
    resource = Path(run(['clang','-print-resource-dir']).stdout.strip())
    work = ROOT/'.work/linux-arm64-kernel-generation'
    work.mkdir(exist_ok=True)
    probe = work/'probe.c'
    command = ['clang','-target','aarch64-linux-gnu','-nostdinc']
    for path in (vendor/'libc/kernel/uapi',vendor/'libc/kernel/uapi/asm-arm64',
                 vendor/'libc/kernel/android/uapi',vendor/'libc/include',resource/'include'):
        command += ['-isystem',path]
    command += ['-std=c11','-Wall','-Wextra','-Werror']
    probe.write_text(INCLUDES)
    ast = json.loads(run([*command,'-fsyntax-only','-Xclang','-ast-dump=json',probe]).stdout)
    records = [n for n in ast['inner'] if n['kind']=='RecordDecl' and
               n.get('name')=='stat' and n.get('completeDefinition')]
    if len(records) != 1:
        raise SystemExit('Missing/ambiguous complete ARM64 kernel stat record')
    shape = record_shape(records[0])
    enum = next(n for n in ast['inner'] if n['kind']=='EnumDecl' and n.get('name')=='BindingKernelConstants')
    constants = {n['name']:folded_value(n) for n in enum['inner'] if n['kind']=='EnumConstantDecl'}
    if any(v is None for v in constants.values()) or len(constants)!=4:
        raise SystemExit('Native kernel macro did not resolve completely')
    probe.write_text(INCLUDES+assertions([shape]))
    checked = run([*command,'-fsyntax-only',probe])
    dependencies = shlex.split(run([*command,'-M',probe]).stdout.split(':',1)[1].replace('\\\n',' '))
    headers, compiler_headers = {}, {}
    with tarfile.open(archive) as stream:
        for filename in dependencies:
            path = Path(filename).resolve()
            if path==probe.resolve():
                continue
            if path.is_relative_to(vendor):
                relative = str(path.relative_to(vendor))
                data = path.read_bytes()
                if data != stream.extractfile(pin['archive']['root']+'/'+relative).read():
                    raise SystemExit('Native kernel header differs from pinned archive: '+relative)
                headers[relative] = digest(data)
            elif path.is_relative_to(resource):
                compiler_headers[str(path.relative_to(resource))] = digest(path.read_bytes())
            else:
                raise SystemExit('Unexpected host header in kernel ABI: '+str(path))
    code = '// Generated only from pinned official Linux ARM64 UAPI C headers.\n'
    code += '// Archive SHA-256: '+pin['archive']['sha256']+'\n\n'
    code += shape['code'].replace('stat :: struct','Kernel_Stat :: struct',1)+'\n\n'
    code += '\n'.join(name+' :: '+str(constants[name])+';' for name in
                      ('SYS_FSTATAT','SYS_FSTAT','CURRENT_DIRECTORY','NOFOLLOW'))+'\n'
    if args.formatter:
        from generate import format_source
        code = format_source(code,args.formatter)
    OUTPUT.parent.mkdir(parents=True,exist_ok=True)
    if args.check:
        if OUTPUT.read_text()!=code:
            raise SystemExit('Stale private Linux ARM64 kernel ABI')
    else:
        OUTPUT.write_text(code)
    report = {'scope':'Actual aarch64-linux-gnu kernel header ABI assertions; not Linux runtime execution',
              'command':[str(x) for x in [*command,'-fsyntax-only',probe]],
              'exit':checked.returncode,'status':'passed','layout':shape,'constants':constants,
              'archive_sha256':pin['archive']['sha256'],'headers':headers,
              'compiler_resource_headers':compiler_headers,'probe_sha256':digest(probe.read_bytes()),
              'generated_source_sha256':hashlib.sha256(code.encode()).hexdigest()}
    args.report.write_text(json.dumps(report,indent=2)+'\n')
    print('Linux ARM64 C kernel layout and syscall definitions verified; runtime unverified')


if __name__=='__main__':
    main()
