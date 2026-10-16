"""Explicit source-reviewed requirements; never classify compiler diagnostics.

Basic.exit is a POSIX/Windows API (basic.md, platform declarations), Io requires
native stack switching, and SDL declares a native platform guard. These lists
describe individual unchanged programs, not entire directories or modules.
"""

NATIVE_EXIT_CASES = frozenset(
    "compiler/" + name
    for name in (
        "arithmetic-overflow-check",
        "array-bounds-check-opt-out",
        "array-literal-view-lifetime",
        "as-member-through-pointer",
        "asm-avx512-masks",
        "asm-evex-decorations",
        "asm-f16c-sha-gfni",
        "asm-gpr-destination",
        "asm-mpsadbw-ifma",
        "asm-scalar-extended",
        "asm-simd-extended",
        "asm-string-compare",
        "asm-vector-instructions",
        "baked-overload-runtime-field",
        "baked-struct-restriction",
        "body-waits-for-inserted-declaration",
        "compile-time-stack-trace",
        "const-string-cast-to-array",
        "decl-type-proc-parens",
        "declare-and-assign-places",
        "discard-parameters",
        "enum-insert-members",
        "for-expansion-forwarded-body",
        "for-expansion-renamed-index",
        "intrinsic-byte-lookup",
        "literal-prefers-float32",
        "must-return-values",
        "operator-ne-from-eq",
        "over-aligned-allocation",
        "param-type-from-param",
        "push-context-defer-pop",
        "rules-must-and-discard",
        "stack-trace-line-through-inline",
        "struct-level-notes",
        "struct-literal-overrides-default-string",
        "struct-literal-without-dot",
        "type-field-constant",
        "type-info-flags",
        "using-discard-struct-constants",
        "using-only-import",
    )
) | frozenset(
    (
        "Extensions/Args/tests/completions",
        "Extensions/Args/tests/help",
        "Extensions/Args/tests/parse",
        "Extensions/Byte_Search/tests/search",
        "Extensions/Jai_Format/tests/golden",
        "Extensions/Json/tests/query",
        "Extensions/Regex/tests/differential",
        "Extensions/Regex/tests/matching",
        "Extensions/Regex/tests/typed",
        "Extensions/Regex/tests/unicode",
        "Extensions/SIMD/tests/vectors",
    )
)

NATIVE_FIBER_CASES = frozenset(
    (
        "extensions/io-accept-burst-diagnostics",
        "extensions/io-allocator-ownership",
        "extensions/io-allocator-zero",
        "extensions/io-configured-production",
        "extensions/io-configured-workers",
        "extensions/io-dependency-workers",
        "extensions/io-fibers-channels",
        "extensions/io-fibers-context",
        "extensions/io-fibers-many",
        "extensions/io-fibers-sleep-cancel",
        "Extensions/Io/tests/pipe",
    )
)

NATIVE_SDL_CASES = frozenset(
    (
        "SDL/tests/sdl-layout",
        "SDL/tests/sdl-rwops",
        "SDL/tests/sdl-wm",
    )
)

# Only these programs execute architectural CPUID/fence/timestamp operations.
# Arithmetic #asm is lowered portably; its syntax alone is not a requirement.
X64_CASES = frozenset(("compiler/lang-asm", "compiler/rules-simd-asm"))

LINUX_URING_CASES = frozenset(
    (
        "extensions/io-uring-dependency-repro",
        "extensions/io-uring-raw",
    )
)


def apply_foundations_capabilities(case):
    """Merge only explicit requirements, preserving other owners' metadata."""
    required = []
    name = case["id"]
    if name in NATIVE_EXIT_CASES:
        required.append("native Basic.exit assertion/termination path")
    if name in NATIVE_FIBER_CASES:
        required.append("native Io providers and fiber stack switching")
    if name in {
        "extensions/http-dependency-consumer",
        "extensions/http-client-overload-lint-repro",
    }:
        required.append("native-only Http App/Client public surface")
    if name == "contracts/basic-slice-resize-and-monotonic-time-contracts":
        required.append("native Basic monotonic/elapsed clock")
    if name in NATIVE_SDL_CASES:
        required.append("native SDL SDK and platform services")
    if required:
        case["unsupported_modes"] = sorted(
            set(case.get("unsupported_modes", ())) | {"wasm-interp", "wasm-native"}
        )
        case["required_capabilities"] = list(
            dict.fromkeys([*case.get("required_capabilities", ()), *required])
        )
    if name in X64_CASES:
        case["required_cpu"] = "x64"
    if name in LINUX_URING_CASES:
        case["required_os"] = "linux"
