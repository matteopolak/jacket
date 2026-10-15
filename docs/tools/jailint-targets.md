# Target-aware jailint source checking

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

## Manifest and full-runner handoff

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

Integration owns the full-lint runner: invoke separate target/context groups,
retain --config/-I and real parameter-providing bootstrap or enclosing program,
collect separate manifests, and require existing strict diagnostics. Do not run
a parameter-required module as an unbound root or import private implementation
from an application wrapper. No module-name lint exceptions are needed or added.
No peer runner files or integration assertion-hook/LSP recovery files were edited.

## Verification and limits

All 38 jailint library/CLI/rule/original target tests pass:

```sh
JAIC_STDLIB="$PWD/target/runtime-configuration-candidate/stdlib" CARGO_TARGET_DIR=target RUSTC_WRAPPER= cargo test --locked -p jailint --all-targets
CARGO_TARGET_DIR=target RUSTC_WRAPPER= cargo clippy --locked -p jailint --all-targets -- -D warnings
```

Logs: target/jailint-target-full-final.log and jailint-target-final-clippy.log.
Original red logs retain missing target flags and missing manifest support.
target/jailint-explicit-context-red.log retains the missing context-option case;
target/jailint-context-unvisited-red.log proves that an unrelated context formerly
returned success without visiting an explicitly requested file. That case now
fails with a coverage diagnostic rather than becoming a false check.
Independent fixtures cover Windows/Linux/macOS x64/ARM64, WASI, host/target
separation, active findings, selected invalid bodies, inactive branches, imports,
loaded file privacy, actual required parameter bindings and failure when absent,
custom source/runtime options, config provenance, parallel roots and fix passes.
An actual child workspace created through implemented private compiler primitives
inherits Windows/ARM64 and records its successful body check; invalid child code
still fails. Explicit context tests exercise real imported parameter bindings
and prove that application private imports remain forbidden.
The first target run had two incorrect test assertions: diagnostic source excerpts
contain inactive names, and a file-private read was incorrectly called as an
export. Assertions now match the actual finding message, and the fixture exports
only the intended public read while retaining a private helper. Failure evidence
is retained in target/jailint-targets-first.log; no compiler visibility was relaxed.
The first missing-context-value test accidentally appended a valid source path,
which the CLI correctly consumed as the value. It now tests a genuinely incomplete
argument vector and requires the missing-value diagnostic; that initial failure
is retained in target/jailint-target-context-missing-value-fixture.log.

Rust formatting/item spacing and host/Windows x64/ARM64 all-target clippy pass.
Windows source-clippy logs are target/jailint-target-windows-{x64,arm64}-clippy.log;
they are not native executable-link evidence. No new standalone Jai implementation
files were added: the intentional finding/rejection cases are embedded original
Rust test inputs, verified by their required lint/compile outcomes rather than
misrepresented as warning-free production modules.

This is source checking on the host, not native Windows/Linux/WASI execution or
full stdlib integration. Compile-time code that actually requires an unavailable
foreign API, the sandbox budget/backend or an unsupported operation still fails;
it is never counted as completed body checking. Polymorphic sketches retain the
compiler's existing limits. No native SDK execution is necessary to verify target
branch selection, but it remains necessary for separate native ABI/runtime claims.
JSON uses the already pinned serde_json version; no dependency version changed.
