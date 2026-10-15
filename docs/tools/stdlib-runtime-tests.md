# Reviewed stdlib tests and independent regressions

The replacement stdlib is tested against reviewed public contracts in
[docs/specs](../specs/README.md), independently authored root contract tests, and
NEW producer behavior suites. Inherited replaced-stdlib API tests and their
failure/coverage baselines were retired with explicit user authorization.
[The migration manifest](../status/stdlib-test-migration.json) records every
original source hash, destination or retirement reason. Retirement does not
establish equivalent exhaustive replacement coverage.

## Sources and provenance

| Family | Location | Oracle |
|---|---|---|
| Public contracts | tests/stdlib-contract/manifest.json | Reviewed public specs; independent author review |
| Compiler semantics and safety | tests/compiler-regressions | Language/compiler contracts and independent safety requirements |
| Extensions | tests/extensions and stdlib/Extensions/*/tests | Existing supported Extensions contracts |
| Native ABI and SDK | tests/native-regressions and tests/native | Actual source-built C/C++ and official SDK behavior |
| Graphics and events | tests/graphics-regressions and tests/window-render | Actual rendering/events, issue 1 wheel and SDK behavior |
| NEW module and producer tests | stdlib/*/tests; tests/portable-stdlib, foundations, platform-io, numerics, bindings, audio, developer-tools | Reviewed public API, private invariants, real native dependencies |

[Independent spec-test coverage notes](../../tests/stdlib-contract/spec-review-coverage.md)
describe bounded root coverage and gaps. No export count or source reachability
percentage establishes completed functionality. Compiler/LSP Rust tests, source
target checks (including unused bodies), SDL3 issue 2, WebGPU, and the unchanged
public application corpus remain independent required gates.

## Run

Build from this repository's sources, with the repository-pinned Rust/LLVM and
Node toolchains. Use the combined production library explicitly:

~~~sh
cargo build --locked -p jaic-cli -p jailint -p jai-language-server
cargo build --locked --release -p jaic-cli
export JAIC_STDLIB="$PWD/stdlib"
target/debug/jaic jaifmt/main.jai -O2 -exe target/jaifmt
python3 tools/build_portable_runtime.py
python3 tools/build_native_libs.py
python3 tools/window_render_native.py
python3 tools/stdlib_runtime.py --jaic target/release/jaic --modes interp,native,wasm-interp,wasm-native --coverage target/stdlib-reachability.txt
python3 tests/stdlib-contract/run.py --jaic "$PWD/target/release/jaic" --stdlib-root "$PWD/stdlib" --mode native --opt O2
python3 tools/stdlib_coverage.py --record target/stdlib-reachability.txt
PYTHONPATH=tools python3 -m unittest tools/test_stdlib_runtime.py tools/test_stdlib_contract_runner.py
~~~

The producer suites have specialized native setup, child processes, fault tests,
arguments and target matrices; their drivers must run as well. On macOS/Linux:

~~~sh
python3 tools/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib --jailint target/debug/jailint --jaifmt target/jaifmt --native-libs artifacts/native-libs/macos-arm64 --font /path/to/real/font.ttf
~~~

Select a different real native-library directory for the host. The aggregator
runs foundations, portable, platform, audio, window and tooling drivers directly
against production. It continues independent suites after failure and retains a
failing aggregate exit. Some producer builders/drivers are currently POSIX or
macOS specific; Windows uses its native CI steps and platform driver, and full
portable/tooling Windows execution is still a gap. No staging overlay is
accepted as production proof.

## Results and configuration

stdlib_runtime.py --filter TEXT selects catalogue IDs; an empty selection is an
error. --modes selects interpreter, native, target-WASM interpreter, or actual
WASI execution; --opt O0|O2 controls generated executables. Source checks and
compile-time-only cases are counted separately from execution. Required
check-fail contracts need nonzero status and the precise reviewed diagnostic;
privacy cases also compile a positive public import baseline. Positive contracts
compare exact stdout. All selected supported failures are fatal. --strict
remains an accepted spelling and cannot permit failures.

Native SDK/graphics families are explicitly unavailable under WASM, reported
separately and never counted passed. Missing SDKs, fonts, helper probes, display
servers and implementation failures on an applicable native host remain failures.
Specialized module tests requiring prepared SDK probes or a font must also run
through their real producer drivers; the generic catalogue does not fabricate
those inputs. Cross-platform source checking is not native execution evidence.
The browser runner uses the same catalogue, reports explicit unavailable foreign
diagnostics separately, and requires precise negative diagnostics/public baselines.

stdlib_coverage.py is informational interpreter reachability: it uses a source
parser and macro-name heuristic. It is neither an exact reflected public API
inventory nor behavior coverage. The obsolete numeric ratchet is removed.

## Changing tests and dependencies

Add reviewed public cases to the contract manifest, with spec provenance and
independent review recorded in the coverage notes. Keep compiler/ABI/Extensions
regressions in their own family; do not invent a stdlib export for a test helper.
A negative fixture needs a nonempty diagnostic_contains and, for privacy, a
positive_baseline. Add specialized cases to their producer driver so correct
native preparation, O0/O2 and target coverage remain required.

Use source-built native dependencies and repository pins. Linux needs native
headers/libraries, Xvfb/Mesa, real fonts and applicable audio backends; macOS uses
official SDKs and actual host graphics/clipboard; Windows requires native MSVC/
LLVM, correct-architecture libraries and Mesa, plus actual native CI. Source-only
or unavailable hardware checks must be stated as such. Corpus vendor library
bodies remain opaque to the integration worker; coordinator runs the original
full application corpus and Focus. Release/publication remains held.

## Independent catalogue channel and compile-only checks

Reviewed contract metadata keeps stdout and stderr separate. A positive with a stdout contract also requires exact stderr (empty unless explicitly supplied); intentional lexer diagnostics are declared, not discarded or merged with success output. Native builds diagnosed without main or without an executable must separately pass strict source checking before being counted compile-time-only. Timeouts and unrelated compiler errors stay failures; compile-time-only never counts as native runtime execution. Cleanup errors propagate. The runner's9 protocol tests include original failed stdout/channel and missing-main/timeout regressions. No inherited skip/coverage baseline is used.

## Durable measured results

Use --report artifacts/<run>.json to retain each completed case/mode, measured wall duration, timeout status, precise negative status and separate platform/compile-only counts. Results are written as independent cases finish; a long case does not hide completed independent results. A failed or interrupted run remains failed/incomplete, and full counts are never projected from focused repairs. The original ac9b8723 Json timeout lacks measured duration because the earlier runner discarded it; its report retains null and the180-second lower bound. A short-deadline runner protocol check is reporting evidence only, never a completed thread functionality test. Mac GetRect widget cases also require the source-built native clipboard restoration guard; restoring plain text alone cannot restore a rich clipboard.
