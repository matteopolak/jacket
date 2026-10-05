# Compiler records (messages, syntax trees, type descriptors)

## What it is

The bridge that lets metaprograms see a workspace's code: `Message_File`, `Message_Import`,
`Message_Typechecked`, the `Code_*` nodes, the `Type_Info_*` descriptors of the *target* program, plus
`compiler_get_nodes` and `compiler_modify_procedure`.

## How it works

- **Why records.** A metaprogram runs in its own compiler's interpreter, which borrows that compiler, and the
  target workspace's types are not the metaprogram's types. So the target's compiler exports neutral
  *records* (`crates/jaic/src/records.rs`): a tag naming a Jai struct (`"Code_Ident"`, `"Type_Info_Struct"`,
  `"Message_Import"`) and named fields — `Int` (floats as bits), `Str`, `Ref` (record id, 0 = null) or a list
  of those. Ids are global to the compilation and records live until it ends.
- **Rust exporter** (`crates/jaic/src/sema/code_export.rs`, state in `Compiler::export`):
  - `export_file_events`: an IMPORT record per module (before its first file) and a FILE record per file.
    Module types: Preload → PRELOAD, Runtime_Support → RUNTIME_SUPPORT, main → MAIN_PROGRAM, others
    UNINITIALIZED. Hidden fields `__module`/`__file` name the module for `add_build_string(.., message)`.
  - `export_typechecked`: every new top-level declaration of a *user* module (not Preload/Runtime_Support,
    not under the system module directory = the directory of `Runtime_Support.jai`) is resolved and exported
    with its type; procedure headers, bodies and struct literals inside it are collected into
    `procedure_headers`/`procedure_bodies`/`structs`, every node made into `subexpressions`. Declarations that
    fail to resolve (or procedures whose signature fails) are skipped, as Jai would report an error instead.
  - Global declarations, procedure parameters/returns (from the signature) and type instantiations that
    evaluate get `type`; local declarations and other expressions do not. Notes after a procedure body
    attach to the declaration and the header. `Code_Node.serial` is the record id.
  - `type_record`: Type_Info records, memoized per `TypeId`. Primitive types carry `__builtin` (their name) so
    the metaprogram uses its own `type_info(int)` etc. — `decl.type == type_info(int)` works. Polymorphic
    struct instances get `specified_parameters`, `constant_storage` and a generic `polymorph_source_struct`;
    `Type` parameters are pointers patched by the Jai side from `__constant_pointers`.
- **Events**: `build::step` takes the record table out of the registry while exporting (resolving may run
  compile-time code) and queues IMPORT/FILE/TYPECHECKED events whose `ints[0]` is the message record.
- **Jai side** (`stdlib/Compiler/records.jai`): `record_struct(id, expected)` allocates the struct named by the
  tag (`RECORD_TYPES`, falling back to the expected pointee type), zero-fills it and fills members by name
  through `Type_Info_Struct`: ints/enums/bools/floats by size, strings, pointers (recursively, memoized so a
  record is one pointer forever), in-place structs and arrays of any of those. `using`/`#as using` members and
  anonymous unions are filled from the same record. `record_of(pointer)` maps back.
- **Speed**: a large metaprogram builds hundreds of thousands of records (Focus: 300k), so the member walk is
  split. `record_plan(info)` flattens a struct type's members once (offset, kind, size, name, type; cached by
  `Type_Info_Struct` pointer). `__jaic_rec_fill` then writes the ints, strings and pointers to records already
  built in one native call, and returns a bit mask of the members Jai must still fill (arrays, in-place
  structs, records not built yet). `__jaic_rec_fill_list` does the same for an array's elements.
  `__jaic_rec_tag_id` numbers tags so `record_type_of` caches the tag's struct type. The pointer-to-record
  index behind `record_of` is built lazily, on the first `record_of` call. Strings handed to Jai point into
  the records' own `Rc<[u8]>` (kept alive in `Workspaces::kept`); tags are static text.
- **compiler_get_nodes**: `Compiler::add_code` mirrors every `Code` value (AST + source snippet) into
  `Interp::codes`; `__jaic_code_nodes` exports it without a compiler (no types or locations) and stores the
  snippet as `__source` on the root. The Jai side remembers each (root, code) pair it hands out
  (`__code_roots` in `workspace.jai`).
- **compiler_get_code**: prints the node tree with `Program_Print` and passes the text to `__jaic_parse_code`,
  which parses it (`build::parse_code_text`) into a new entry of `Interp::codes` and records in
  `Interp::made_codes` which code's scope it takes (the code the root came from, else
  `code_to_copy_scope_from`). The compiler adopts such codes when it reads a `Code` value back from compile-time
  code or adds a code of its own (`Compiler::adopt_made_codes`), re-parsing the text as a registered source so
  diagnostics can point into it. Node edits the printer cannot express (see
  [program-print](../specs/stdlib/README.md)) are lost.
- **compiler_modify_procedure**: Jai sends the record ids of `body.block.statements`; they are queued on the
  workspace and applied at its next step (`Compiler::modify_procedure`): statements exported from that body map
  back to their AST, `#code` roots are re-parsed from `__source` inside a dummy procedure. The procedure's
  `ProcLit` is replaced and, if the body was already lowered, lowered again into the same function.
- **TYPECHECKED for procedures**: a procedure whose body is not lowered yet is reported with a null
  `body_or_null` and queued in `ExportState::pending_bodies`. Each later `export_typechecked` reports the
  queued bodies that have been lowered since (records carry local declaration types) and patches the header's
  `body_or_null`. Bodies the program never reaches are never lowered, so their errors never surface — as in Jai.
- **Enums**: a top-level enum declaration's `Code_Enum.external_type` is its `Type_Info_Enum`.
- **Phases**: when a workspace runs out of sources, `build.rs::step` first lowers everything reachable
  (`Compiler::lower_reachable`, lenient: a failing body stays queued). If that reports new declarations or bodies (a body may declare more through
  `#insert,scope(...)`), the metaprogram gets another `TYPECHECKED_ALL_WE_CAN` and may add code; only a
  lowering round that reports nothing new ends in `finish_program` and code generation.

## How to change it

- New node kind: add a case in `Exporter::expr`/`stmt` building a record with the `Code_*` member names, and
  add the struct to `RECORD_TYPES` in `records.jai` if it is reached through a base pointer.
- New field: just add it to the record — the Jai filler matches names, unknown names are ignored, missing
  fields stay zero. Keep names identical to `stdlib/Compiler/nodes.jai`/`stdlib/prelude/module.jai`.
- Only top-level statement lists can be modified; nested blocks would need their statement records mapped too.
- Gotcha: anything that resolves in the exporter can run compile-time code; never hold the registry borrow.

## Configuration

None. Exports happen only for intercepted workspaces.

## Dependencies

`sema` (entity resolution, signatures, `eval_type`, struct layouts), `build.rs` (events, `__jaic_rec_*`,
`__jaic_code_nodes`, `__jaic_modify_procedure`, `__jaic_workspace_add_string_to_module`), the parser (re-parsing
inserted code). Test: `tests/stdlib/compiler-typechecked-messages.jai`.
