#!/usr/bin/env python3
"""Per-CPU Linux POSIX shapes, symbols and constants from that CPU's own libc headers.

`docs/specs/stdlib/modules/posix.md` records the x86-64 and arm64 Linux ABI. Every other
Linux CPU differs in `long`/pointer width, struct layout, kernel constants and (with 64-bit
`time_t` on 32-bit CPUs) the symbol a call links to. This script asks clang, inside the cross
container (`tools/dev/linux_x64.sh sysroots`), what the target's glibc headers declare for each
name the spec exports, and writes the differences to
`stdlib/internal/bindings/vendor/linux-posix-<arch>.json`, which `generate.py` turns into the
`ARCHITECTURE == .<ARCH>` declarations of `stdlib/POSIX/declarations.jai`.

    python3 tools/native_bindings/vendor_linux_posix.py i686
    python3 tools/native_bindings/vendor_linux_posix.py i686 --check
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

from conditions import evaluate
from native_bindings.spec import parse_spec
import linux_cabi as cabi
from toolkit.repo import ROOT

SPEC = ROOT/'docs/specs/stdlib/modules/posix.md'
OUT = ROOT/'stdlib/internal/bindings/vendor'
WORK = ROOT/'.work/linux-posix'
CONTAINER = ROOT/'tools/dev/linux_x64.sh'

# name: (clang triple, sysroot in the cross container, `Architecture` member, bytes in `long`)
ARCHES = {
    'i686': ('i686-linux-gnu', '/usr/i686-linux-gnu', 'X86', 4),
    's390x': ('s390x-linux-gnu', '/usr/s390x-linux-gnu', 'S390X', 8),
    'powerpc64': ('powerpc64-linux-gnu', '/usr/powerpc64-linux-gnu', 'POWERPC64', 8),
    'armv7': ('arm-linux-gnueabihf', '/usr/arm-linux-gnueabihf', 'ARM', 4),
}

# Spec names whose C counterpart is spelled differently.
C_NAMES = {'sigaltstack_t': 'stack_t', 'sigset_t': '__sigset_t'}

PROBE_HEADERS = '''aio.h arpa/inet.h assert.h dirent.h dlfcn.h errno.h fcntl.h fnmatch.h glob.h grp.h
ifaddrs.h langinfo.h libgen.h link.h locale.h mqueue.h net/if.h netdb.h netinet/in.h
netinet/tcp.h poll.h pthread.h pwd.h sched.h semaphore.h setjmp.h signal.h spawn.h stdio.h
stdlib.h string.h sys/epoll.h sys/eventfd.h sys/file.h sys/inotify.h sys/ioctl.h sys/ipc.h
sys/mman.h sys/msg.h sys/prctl.h sys/random.h sys/resource.h sys/select.h sys/sem.h
sys/sendfile.h sys/shm.h sys/signalfd.h sys/socket.h sys/stat.h sys/statvfs.h sys/syscall.h
sys/sysinfo.h sys/time.h sys/timerfd.h sys/times.h sys/types.h sys/uio.h sys/un.h sys/user.h
sys/utsname.h sys/wait.h syslog.h termios.h time.h ucontext.h unistd.h utime.h wchar.h
wordexp.h'''.split()
PROBE_PRELUDE = '#define _GNU_SOURCE 1\n#define _FILE_OFFSET_BITS 64\n#define _TIME_BITS 64\n'


def linux_declarations(spec):
    def linux(condition):
        try:
            return evaluate(condition or 'true', dict(OS='LINUX', CPU='X64'))
        except ValueError:
            return False
    return [d for d in spec.declarations if linux(d.condition)]


def container(command):
    result = subprocess.run([str(CONTAINER), 'cross-exec', '--', 'sh', '-c', command],
                            capture_output=True, text=True)
    if result.returncode:
        sys.exit('cross container command failed (is it up? tools/dev/linux_x64.sh cross-up):\n'
                 + command + '\n' + result.stderr[-2000:])
    return result.stdout


def dump(arch):
    """Clang's JSON AST, record layouts and macro list for the probe, cached under `.work`."""
    triple, sysroot, _, _ = ARCHES[arch]
    directory = WORK/arch
    directory.mkdir(parents=True, exist_ok=True)
    probe = directory/'probe.c'
    probe.write_text(PROBE_PRELUDE + ''.join('#include <%s>\n' % h for h in PROBE_HEADERS))
    flags = '--target=%s -isystem %s/include' % (triple, sysroot)
    ast, layouts, macros = (directory/n for n in ('ast.json', 'layouts.txt', 'macros.txt'))
    container('cd %s && clang %s -Xclang -ast-dump=json -fsyntax-only probe.c > ast.json'
              ' && clang %s -Xclang -fdump-record-layouts -fsyntax-only probe.c > layouts.txt'
              ' && clang %s -dM -E probe.c > macros.txt' % (directory, flags, flags, flags))
    return json.loads(ast.read_text()), layouts.read_text(), macros.read_text()


def evaluate_constants(arch, names, macros):
    """{name: value} of the integer macros and enum constants clang can fold for `names`."""
    triple, sysroot, _, _ = ARCHES[arch]
    defined = {m[1] for m in re.finditer(r'#define (\w+) ', macros)}
    wanted = sorted(n for n in names if n in defined)
    directory = WORK/arch
    for _ in range(4):
        text = PROBE_PRELUDE + ''.join('#include <%s>\n' % h for h in PROBE_HEADERS)
        text += 'enum {\n' + ''.join('    __v_%s = (%s),\n' % (n, n) for n in wanted) + '};\n'
        (directory/'constants.c').write_text(text)
        out = subprocess.run([str(CONTAINER), 'cross-exec', '--', 'sh', '-c',
                              'cd %s && clang --target=%s -isystem %s/include -Xclang -ast-dump=json '
                              '-fsyntax-only constants.c 2> errors.txt > constants.json' % (directory, triple, sysroot)],
                             capture_output=True, text=True)
        errors = (directory/'errors.txt').read_text()
        bad = {int(m[1]) for m in re.finditer(r'constants\.c:(\d+):\d+: error:', errors)}
        if not bad:
            break
        base = text.count('\n') - len(wanted) - 1
        dropped = {wanted[line - base - 1] for line in bad if 0 < line - base <= len(wanted)}
        if not dropped:
            sys.exit('unexpected constant probe errors:\n' + errors[-1500:])
        wanted = [n for n in wanted if n not in dropped]
    ast = json.loads((directory/'constants.json').read_text())
    values = {}
    for node in ast['inner']:
        if node['kind'] == 'EnumDecl':
            for member in node.get('inner', []):
                if member['kind'] == 'EnumConstantDecl' and member['name'].startswith('__v_'):
                    value = cabi.folded(member)
                    if value is not None:
                        values[member['name'][4:]] = value
    return values


def types_of(signature):
    """The parameter and result types of a Jai `name :: (a: T, ...) -> R` signature."""
    match = re.fullmatch(r'\w+\s*::\s*\((.*)\)\s*->\s*(.*?)\s*(?:#foreign.*)?', signature.strip())
    if not match:
        return None
    params, depth, current, parts = match[1], 0, '', []
    for char in params:
        depth += char in '(['
        depth -= char in ')]'
        if char == ',' and depth == 0:
            parts.append(current)
            current = ''
        else:
            current += char
    if current.strip():
        parts.append(current)
    return [re.sub(r'^\s*\w+:\s*', '', p).strip() for p in parts], match[2].strip()


def build(arch):
    spec = parse_spec(SPEC.read_text())
    declarations = linux_declarations(spec)
    names = {d.name for d in declarations}
    _, _, architecture, long_bytes = ARCHES[arch]
    ast, layout_dump, macros = dump(arch)
    keep = set(names)
    enums = {d.name for d in declarations if re.search(r'::\s*enum\b', d.code)}
    renames = {m[2]: m[1] for m in re.finditer(r'#define (\w+) (\w+)\.tv_sec\b', macros)}
    # `#define sa_handler __sigaction_handler.sa_handler`: the spec's record holds those members
    # directly. (Longer paths, like `si_pid`, name members of records the spec leaves nested.)
    reached = {m[1] for m in re.finditer(r'#define \w+ (\w+)\.\w+\s*$', macros, re.M)} \
        - set(renames.values())
    ctx = cabi.Context(ast, long_bytes, keep, renames, reached)
    # Records the spec leaves opaque (`struct {}`) have no layout to get wrong.
    opaque = {}
    for shape in spec.shapes:
        opaque[shape.name] = opaque.get(shape.name, True) and bool(re.fullmatch(r'\w+\s*::\s*struct\s*\{\s*\}', shape.code.strip()))
    sizes = cabi.layouts(layout_dump)
    result = {'arch': arch, 'architecture': architecture, 'triple': ARCHES[arch][0],
              'typedefs': {}, 'shapes': {}, 'functions': {}, 'constants': {},
              'layouts': {}, 'uncovered': []}
    for d in declarations:
        name, code = d.name, d.code.strip()
        bound = 'CPU' in d.condition  # declared only for the CPUs the spec lists
        try:
            if '#foreign' in code and name in ctx.functions:
                params, ret, symbol = cabi.function_declaration(ctx, name)
                spelled = types_of(code)
                if spelled is not None and len(spelled[0]) == len(params):
                    # The spec names an enum where the header has `int`: same width, keep the name.
                    for i, (wanted, got) in enumerate(zip(spelled[0], params)):
                        head, _, kind = got.partition(': ')
                        if wanted in enums and kind in ('s32', 'u32'):
                            params[i] = '%s: %s' % (head, wanted)
                        # `int fds[2]` is the pointer `int *` to the header; the spec keeps the array.
                        array = re.fullmatch(r'\*\[\d+\]\s*(.+)', wanted)
                        if array and kind == '*' + array[1]:
                            params[i] = '%s: %s' % (head, wanted)
                header = [re.sub(r'^\w+:\s*', '', p) for p in params]
                if bound or spelled is None or (spelled[0], spelled[1].rstrip()) != (header, ret) or \
                        symbol != name:
                    library = re.search(r'#foreign\s+(\w+)', code)[1]
                    suffix = '' if symbol == name else ' "%s"' % symbol
                    result['functions'][name] = '%s :: (%s) -> %s #foreign %s%s' % (
                        name, ', '.join(params), ret, library, suffix)
            elif re.search(r'::\s*(struct|union)\b', code):
                declared = cabi.shape(ctx, C_NAMES.get(name, name))
                if declared is None:
                    if not opaque.get(name):
                        result['uncovered'].append(name)
                    continue
                result['shapes'][name] = declared
                key = ctx.record_key(C_NAMES.get(name, name))
                if key in sizes:
                    result['layouts'][name] = list(sizes[key])
            elif re.fullmatch(r'\w+\s*::\s*(?:[su]\d+|\*?\w+|\[\d+\]\s*\w+);?', code) and name in ctx.typedefs:
                declared = cabi.typedef_declaration(ctx, name)
                if declared.rstrip(';') == '%s :: %s' % (name, name):
                    # `typedef union { ... } name;`: the spec's pointer or scalar is a record here.
                    result['shapes'][name] = cabi.shape(ctx, name)
                elif bound or declared.rstrip(';') != code.rstrip(';'):
                    result['typedefs'][name] = declared
        except cabi.Unsupported as why:
            result['uncovered'].append('%s (%s)' % (name, why))
    numeric = {d.name for d in declarations if re.search(r'::\s*-?(?:0x[0-9a-fA-F]+|\d+)\b', d.code)}
    values = evaluate_constants(arch, numeric, macros)
    for d in declarations:
        match = re.fullmatch(r'(\w+)\s*::\s*(-?(?:0x[0-9a-fA-F]+|\d+));?', d.code.strip())
        if match and match[1] in values and ('CPU' in d.condition or int(match[2], 0) != values[match[1]]):
            result['constants'][match[1]] = values[match[1]]
    result['uncovered'] = sorted(set(result['uncovered']))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument('arch', choices=sorted(ARCHES))
    parser.add_argument('--check', action='store_true', help='fail if the committed file is stale')
    args = parser.parse_args()
    text = json.dumps(build(args.arch), indent=1, sort_keys=True) + '\n'
    path = OUT/('linux-posix-%s.json' % args.arch)
    if args.check:
        if not path.exists() or path.read_text() != text:
            sys.exit('%s is stale; run tools/native_bindings/vendor_linux_posix.py %s' % (path, args.arch))
        return
    path.write_text(text)
    print('wrote', path.relative_to(ROOT))


if __name__ == '__main__':
    main()
