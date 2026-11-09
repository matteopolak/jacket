# Dead-code elimination

Which declarations jaic type-checks when nothing the program runs uses them. `Build_Options.dead_code_elimination` (or `-no_dce`) picks the mode. Only checking depends on it: compiled output never contains unreached code.

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

The same declarations in an imported module go unchecked until something calls them, so a module can carry stale or other-platform helpers without breaking importers.

## How it works

- The program's own files are the files given to the compiler (or a workspace's `add_build_file`/`add_build_string`) and everything they `#load`. A module's files are what `#import`, `#import,file` and `#import,dir` load, plus what those `#load` {#dce.7}.
- Under `.MODULES_ONLY`, in the program's files every global (type and initializer) and constant is checked {#dce.1}, and so is every procedure body unless it is polymorphic or a macro, including nested ones {#dce.2}.
- A struct constant is checked after its struct is laid out, so it can name the struct's members (`PAD :: size_of(type_of(info))`) {#dce.13}.
- What those bodies call is checked like any called code, so a module procedure called only by an unreferenced program procedure is checked too {#dce.3}.
- Polymorphic procedures are checked per instance and macros per expansion, so never-used ones are never checked {#dce.4}.
- An inactive `#if` branch declares nothing, so nothing in it is checked {#dce.5}.
- `#run`s inside bodies checked this way run, after the top-level `#run`s {#dce.11}. In modules, a body nothing reaches is never checked and its `#run`s never run {#dce.6}.
- `.NONE` applies the program-file rules to every module {#dce.8}. `.ALL` checks only what the program reaches {#dce.9}.
- In every mode, every non-polymorphic struct and union is laid out, so a member of an undefined type, or a default of the wrong type, is an error even in an unused struct {#dce.10}.
- Compiled output holds only functions that the exported entry points, globals and runtime information reach; code lowered only to check something is dropped {#dce.12}.
- Declarations are visited in creation order, so which error is reported does not depend on hashing.

## Differences from Jai

- Jai also checks module declaration headers, globals and constants under `.MODULES_ONLY` and `.ALL`; jaic checks them only when used (structs excepted).
- Jai compiles unreferenced procedures into the executable; jaic drops them.
- Bodies checked by this pass come after the last `Message_Typechecked`, so a metaprogram never sees them.

## How to change it

- A new kind of unreferenced declaration: extend `unreferenced_entity_wanted` or `check_unreferenced_proc` in the `check_unreferenced` pass (`sema/driver.rs`). It runs after `placeholders_final`, so it must not wait on code a metaprogram adds.
- Which files count as the program's: `checks_unreferenced_in`.
- A new IR instruction that refers to a function, global or foreign symbol must be taught to `drop_unreferenced_code`.
- A new sweep failure after a change here is usually a latent sema bug in never-checked code.
