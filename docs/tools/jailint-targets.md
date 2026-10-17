# Target-aware jailint source checking

Import roots supplied by -I/-import_dir (or driver compiler_options.import_paths)
are resolved relative to the invocation working directory, consistent with jaic.
They are anchored before each source/module root is compiled, including loaded
fragments and child workspaces. Missing roots stay missing; source-local directories
with the same relative name cannot accidentally supply a module. Absolute roots
retain their behavior. Regression tests cover relative/absolute/no-I controls, `./` and `..` paths,
loaded fragment context, source-local shadows and invalid imported bodies. No
warning is relaxed and failed imports/bodies remain incomplete/fatal.

Jailint accepts the compiler's source-checking target flags:

```sh
jailint -os windows -cpu arm64 -D warnings path/to/module
jailint -os linux -cpu x64 -D warnings path/to/module
jailint -os macos -cpu arm64 -D warnings path/to/module
jailint -target wasm64-unknown-wasi -D warnings path/to/module
jailint -os windows -cpu x64 --manifest target/windows-lint.json path/to/module
jailint -os windows -cpu arm64 --context path/to/configured-program.jai path/to/module
```

These select real compiler Options before checking; they do not rewrite OS/CPU
constants, substitute host guards or invoke a native backend. BUILD_OS/BUILD_CPU
continue to identify the executing compiler. No flags preserves host behavior.
OS names are linux/windows/macos/wasm; CPU names are x64/x86_64 and arm64/aarch64.
Wasm selects the compiler's CUSTOM CPU unless explicitly overridden. A triple
overrides OS/CPU flags, matching jaic. Unknown architecture/OS names diagnose as
usage errors rather than silently becoming Linux/X64. Supported architecture/OS
components are mapped for source checking; this does not validate linker/SDK
availability or establish support for another pointer ABI.

As in jaic checking, Windows selected from another OS defaults to x64 and the GNU
C ABI unless an explicit triple selects MSVC. Use x86_64-pc-windows-msvc or
aarch64-pc-windows-msvc when checking the Microsoft ABI. C long-double selection
uses the compiler's existing target mapping. WASI is selected by its wasm64 triple.

driver::Options now carries compiler_options and optional source in addition to all existing root,
import, library, config, jobs and verbosity fields. Each root clones these options,
preserving configured target, runtime switches, import paths and an explicit
preload. Context setup adds root-local search paths and fills the canonical
bootstrap only when absent. BuildEnv clones those same options for child
workspaces; explicit metaprogram configuration still governs its own workspaces.
No application/private import permission or required module binding is relaxed.
--context selects a real enclosing program as that source; requested paths still
select the files to lint. This supports a parameterized module imported by that
program without re-importing it with invented defaults. Context paths must name
files. Context provenance remains the program's own: listing a private library
file never authorizes an application context to import it.
If an explicitly requested source file is absent from that context, checking
fails instead of returning success for an unrelated program. Directory requests
can contain target-inactive loads; their unvisited entries remain explicit in
the manifest and are not counted as coverage.
Loaded files are checked in their actual containing context. Parallel workers and
all fix passes reuse the same configuration. Config.source remains intact.

Compiler and IDE body diagnostics remain unconditional CLI failures. A failed
IDE body check now also marks the context incomplete for lint rules requiring
complete uses. Every actual compiler context visits its requested files, rather
than treating an earlier workspace's visit as coverage for a later target.
Identical findings are deduplicated after checking, not by skipping bodies.

## Manifest

--manifest writes version-1 JSON even when source compilation fails. It reports:

- kind=source-check, phase=initial (before any --fix edits);
- invocation target OS/CPU and optional triple, config_source and root count;
- contexts with their root, actual compiler OS/CPU, complete flag and visited_files;
- unvisited_files from requested/load-discovered sources absent from all contexts;
- incomplete roots with the retained first compiler error.

Root/visited paths are canonical; invocation source/config provenance retains
the selected paths. complete means the compiler and IDE
checks for that context returned without diagnostics; it does not mean no lint
findings or all possible generic instantiations were compiled. A visited file in
an incomplete context is partial evidence. Unvisited conditional loads are not
counted as checked, and must be covered by another appropriate target/context.
Manifest write failures return a usage/configuration error. Ordinary incomplete
compilation or denied findings still returns 1; usage/configuration errors return
2. --fix retains existing initial-pass exit/report behavior; the manifest labels
that phase explicitly.

To lint a whole project, invoke separate target/context groups, retain
--config/-I and the real parameter-providing bootstrap or enclosing program,
collect the separate manifests, and require strict diagnostics. Do not run a
parameter-required module as an unbound root or import private implementation
from an application wrapper.
