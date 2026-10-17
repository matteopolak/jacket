# Dead-code elimination

## What it is

Which declarations jaic type-checks when nothing the program runs uses them. `Build_Options.dead_code_elimination` (and `-no_dce`) picks the mode. Only checking depends on it: compiled output never contains unreached code in any mode.

| Mode | Checked even when unreferenced |
|---|---|
| `.MODULES_ONLY` (default) | Everything in the program's own files |
| `.NONE` (`-no_dce`) | Everything, modules included |
| `.ALL` | Only struct layouts; the rest only if reached |

```jai
y: int = "a";                  // error even though main never uses y
bad :: () { x: int = "a"; }    // error: bodies in the program's files are checked
main :: () { print("hi\n"); }
```

The same declarations in an imported module go unchecked until something calls them, so a module can carry other-platform or stale helpers without breaking importers.

## How it works

The program's own files are those in the main module (`Scope::module == Compiler::main_module`): files given to the compiler, a workspace's `add_build_file`/`add_build_string`, and what they `#load`. Under `.MODULES_ONLY`, in those files every global, constant and non-polymorphic, non-macro procedure body is checked (including struct-body and nested procedures); struct constants are checked after layout. What those bodies call is checked as called code. Polymorphic procedures and macros are checked per instance or expansion. Inactive `#if` branches declare nothing. `#run` inside checked bodies runs, after the top-level `#run`s. Every non-polymorphic struct and union is laid out in every mode, so a member of an undefined type is always an error.

Implementation:

1. `finish_program` (`sema/driver.rs`) lowers what is reachable, lays out structs and fills runtime info.
2. `check_unreferenced` walks `Compiler::entities` and `Compiler::procs` in creation order until nothing new appears. `unreferenced_entity_wanted` picks unresolved declarations in checked modules (`checks_unreferenced_in`); `check_unreferenced_proc` picks procedures with no target (settling `refresh_implicit_poly`, skipping polymorphic ones; bodiless `#foreign`/`#compiler` procedures only get their signature checked). Anything inside an instance or expansion is skipped (`inside_instance`).
3. Before it lowers anything it records `Compiler::unreferenced_from`; `prepare_compiled_output` then calls `drop_unreferenced_code`, which walks from older functions and globals (plus exports) and drops newer unreached functions, initializers and foreign symbols. The interpreter keeps everything.

Declarations are visited in creation order, so the reported error does not depend on hashing. The language server publishes the first error as a `jaic` diagnostic with code `check`.

### Differences from Jai

- Jai also checks module declaration headers, globals and constants under `.MODULES_ONLY` and `.ALL`; jaic checks them only when used (structs excepted).
- Jai compiles the program's unreferenced procedures into the executable; jaic drops them.
- Bodies checked by this pass come after the last `Message_Typechecked`, so a metaprogram never sees them.

## How to change it

- New kind of unreferenced declaration: extend `unreferenced_entity_wanted` or `check_unreferenced_proc`. It runs after `placeholders_final`, so it must not wait for code a metaprogram adds.
- Which files count as the program's: `checks_unreferenced_in`.
- A new IR instruction referring to a function, global or foreign symbol must be taught to `drop_unreferenced_code`.
- A new sweep failure after a change here is usually a latent sema bug in never-checked code.

Configuration: `set_build_options` in `stdlib/internal/compiler-api/workspaces.jai`; `sema::Options::dead_code`; `build::BuildSettings::dead_code_elimination`. Tests: `tests/stdlib/dead-code-elimination.jai`, `tests/corpus/negative/dce-*.jai`, `dead_code_elimination_flag` in `crates/jaic-cli/tests/cli.rs`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- The program's own files are those in the main module: the files given to the compiler (`jaic file.jai`, a workspace's `add_build_file` and `add_build_string`) and every file they `#load`. A module's files are what `#import`, `#import,file` and `#import,dir` load (the program's `modules/` folder too), and every file such a module `#load`s {#dce.7}. Sema tells them apart by `Scope::module == Compiler::main_module`.
- Every global variable (its type and initializer) and every constant is checked {#dce.1}.
- Every procedure body is checked unless the procedure is polymorphic or a macro, including procedures declared in a struct body or inside another procedure's body {#dce.2}.
- A struct constant is checked after its struct is laid out, as a use of the struct would, so it may name the struct's members (`PAD :: size_of(type_of(info))`), also in a struct declared inside another struct {#dce.13}.
- What those bodies call is checked like any called code, so a module procedure that only an unreferenced procedure of the program calls has its body checked {#dce.3}.
- A polymorphic procedure's body is checked per instance and a macro's per expansion, so one that is never instantiated or expanded is never checked {#dce.4}.
- An inactive `#if` branch declares nothing, so nothing in it is checked {#dce.5}.
- `#run` directives inside a body checked this way run, since checking a body evaluates its compile-time code {#dce.11}. They run when the program has been compiled, after the top-level `#run`s.
- In modules, a procedure body nothing reaches is never checked, and the `#run`s inside it never run {#dce.6}.
- `.NONE` applies the program-file rules to every module as well {#dce.8}. `.ALL` checks only what the program reaches, which is how jaic behaved before the modes existed {#dce.9}. In every mode, every non-polymorphic struct and union declaration is laid out, so a member of an undefined type is an error even in a struct nothing uses {#dce.10} (see [sema: declarations](../compiler/sema.md)).
- Compiled output contains only the functions that the program's exported entry points, its globals and its runtime information reach. Code lowered only to check something unreferenced is dropped {#dce.12}.
