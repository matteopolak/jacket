# Dead-code elimination

## What it is

Which declarations jaic type-checks when nothing the program runs uses them. `Build_Options.dead_code_elimination` (and the `-no_dce` command-line flag) picks the mode. Only checking depends on it: compiled output never contains code the program does not reach, in any mode.

| Mode | Checked even when unreferenced |
|---|---|
| `.MODULES_ONLY` (default) | Everything declared in the program's own files |
| `.NONE` (`-no_dce`) | Everything, modules included |
| `.ALL` | Nothing but struct layouts: only what the program reaches is checked |

So this program fails to compile, although `main` never touches `y` or `bad`:

```jai
#import "Basic";
y: int = "a";                  // error: type mismatch: expected s64, found string
bad :: () { x: int = "a"; }
main :: () { print("hi\n"); }
```

The same declarations in an imported module go unchecked until something calls them. That is what lets a module carry procedures for other platforms or stale helpers without breaking every program that imports it.

## How it works

### Program files and module files

The program's own files are those in the main module: the files given to the compiler (`jaic run/check/build file.jai`, a workspace's `add_build_file` and `add_build_string`) and every file they `#load`. A module's files are what `#import`, `#import,file` and `#import,dir` load (the program's `modules/` folder too), and every file such a module `#load`s {#dce.7}. Sema tells them apart by `Scope::module == Compiler::main_module`.

### The rules

Under `.MODULES_ONLY`, in the program's own files:

- Every global variable (its type and initializer) and every constant is checked {#dce.1}.
- Every procedure body is checked unless the procedure is polymorphic or a macro, including procedures declared in a struct body or inside another procedure's body {#dce.2}.
- What those bodies call is checked like any called code, so a module procedure that only an unreferenced procedure of the program calls has its body checked {#dce.3}.
- A polymorphic procedure's body is checked per instance and a macro's per expansion, so one that is never instantiated or expanded is never checked {#dce.4}.
- An inactive `#if` branch declares nothing, so nothing in it is checked {#dce.5}.
- `#run` directives inside a body checked this way run, since checking a body evaluates its compile-time code {#dce.11}. They run when the program has been compiled, after the top-level `#run`s.

In modules, a procedure body nothing reaches is never checked, and the `#run`s inside it never run {#dce.6}.

`.NONE` applies the program-file rules to every module as well {#dce.8}. `.ALL` checks only what the program reaches, which is how jaic behaved before the modes existed {#dce.9}. In every mode, every non-polymorphic struct and union declaration is laid out, so a member of an undefined type is an error even in a struct nothing uses {#dce.10} (see [sema: declarations](../compiler/sema-polymorphism-and-declarations.md)).

Compiled output contains only the functions that the program's exported entry points, its globals and its runtime information reach. Code lowered only to check something unreferenced is dropped {#dce.12}.

### Implementation

1. `finish_program` (`sema/driver.rs`) lowers what is reachable, lays out the declared structs and fills the runtime information, as before.
2. `check_unreferenced` then walks `Compiler::entities` and `Compiler::procs` in creation order, repeating until a round finds nothing new (checking creates more: nested procedures, `#insert`ed declarations):
   - `unreferenced_entity_wanted` picks unresolved declarations whose module is checked (`checks_unreferenced_in`). That means module and file scope declarations, struct constants, and procedure-literal constants declared inside bodies. `resolve_entity` checks them.
   - `check_unreferenced_proc` picks procedures with no target yet that are neither macros nor instances. It settles implicit polymorphism (`refresh_implicit_poly`) and skips polymorphic ones. A procedure without a body (`#foreign`, `#compiler`) only gets its signature checked, because a target would add a foreign symbol and link its library. Others get `proc_func` and `drain_bodies`, the same lowering a call triggers.
   - Anything inside a polymorphic instance, a macro expansion or a polymorphic struct instance is skipped (`inside_instance`): it is checked as that instance is used.
3. Before `check_unreferenced` lowers anything it records `Compiler::unreferenced_from` (how many functions, globals and foreign symbols exist). `prepare_compiled_output` calls `drop_unreferenced_code`, which walks from every function and global older than the mark (plus exported ones) through calls, function and global addresses and relocations. Newer functions it does not reach become `None` (backends skip them), newer globals keep their size but lose their initializer, and newer foreign symbols stop pulling in their libraries. The interpreter (`jaic run`) keeps everything; it only runs what is called.

The first error ends compilation as usual. Declarations are visited in creation order, which follows source order within a file, so the reported error does not depend on hashing.

Errors from this pass are ordinary compile errors. The language server publishes the first error of the program as a `jai-check` diagnostic (see [language server](../compiler/language-server.md)), so an unused procedure's error shows up in the editor.

### Differences from Jai

- Jai also checks every declaration's header, global and constant in modules under `.MODULES_ONLY` and `.ALL`. jaic checks module declarations when something uses them (structs excepted).
- Jai compiles the program's own unreferenced procedures into the executable. jaic drops them.
- Bodies checked by this pass come after the last `Message_Typechecked`, so a metaprogram never sees them ([compiler records](../metaprogramming/compiler-records.md)).

## How to change it

- To check another kind of unreferenced declaration, extend `unreferenced_entity_wanted` or `check_unreferenced_proc`. Whatever they resolve runs after `placeholders_final`, so it must not wait for code a metaprogram adds.
- To change which files count as the program's own, change `checks_unreferenced_in`.
- If a new IR instruction refers to a function, global or foreign symbol, teach `drop_unreferenced_code` about it, or the referenced object may be dropped from output.
- A new failure in the sweep after a change here is usually a latent sema bug in code that was never checked before. Real Jai projects compile with Jai, which checks their own files.

## Configuration

- `Build_Options.dead_code_elimination` (`stdlib/Compiler/options.jai`): `NONE`, `ALL`, `MODULES_ONLY` (default). A metaprogram sets it per workspace. `set_build_options` sends it only when it changed, so options that never mention it keep the command line's choice. A program may set it for its own workspace too; `finish_program` reads the setting last.
- `-no_dce` (`jaic run|check|build`): `.NONE` for the program and the workspaces its metaprogram creates.
- `sema::Options::dead_code` (`DeadCode`) is the embedder's default; `build::BuildSettings::dead_code_elimination` carries a workspace's setting.

## Dependencies

- `sema/driver.rs`: `check_unreferenced`, `drop_unreferenced_code`.
- `sema/procs.rs`: `proc_func`, `drain_bodies`, `refresh_implicit_poly`.
- `build.rs`: the `dead_code_elimination` workspace option and `dead_code_setting`.
- Tests: `tests/stdlib/dead-code-elimination.jai`, `tests/corpus/negative/rule-dce-*.jai` and `dce-*.jai`, `dead_code_elimination_flag` in `crates/jaic-cli/tests/cli.rs`, `unreferenced_code_is_not_compiled` in `crates/jaic-cli/tests/native.rs`, `type_error_in_an_unused_procedure_is_published` in `crates/jai-language-server/tests/semantic.rs`.
