# Note readers

Editor compiles record which compile-time code read each note (`@Traced`, `foo :: () @PrintLike`). `jailsp` answers go to definition on a note with those places. `jailint`'s `unused_note` rule reports notes nothing read.

## How it works

1. **Watch.** While `Compiler::ed` records, `call_thunk` turns on the interpreter's note watch (`interp/note_watch.rs`). It stays `None` in a batch compile.
2. **Register.** Each note's text bytes are registered with the note's place: its file as `program_path` spells it, and the line and byte column where it starts and ends. Every note's text has its own allocation, so each byte range belongs to one note. Two paths hand out notes:
   - the record builder (`record_build.rs`) registers a `Code_Note`'s text when it gives a metaprogram the record;
   - type info (`Type_Info_Struct.notes`, and `Type_Info_Struct_Member.notes` of fields and constants): in editor compiles `set_notes_view` puts each note's text in a global of its own (`Out::note_global`, listed in `Program::note_globals`), and `Interp::global_addr` registers it when it makes the global.
3. **Read.** An access that touches registered bytes is a read:
   - a load in the dispatch loop that watches blocks (`run_ops::<true>`; a watch switches the run to it), or in the IR fallback;
   - a copy (`Copy`, `MemCopy`, the `memcpy` intrinsic);
   - a `memcmp` that found the bytes equal.

   The read keeps the interpreter's location and its callers' locations.
4. **Record.** After each run, `Compiler::ide_note_reads` turns reads into `NoteReader`s in `IdeFacts::note_readers`. The site is the innermost location in a recorded file, so a read inside a library helper lands on the user's call.
5. **Share.** The note is usually in a workspace compiler and the reading code in the one running the metaprogram. After a check, `jaic::ide::share_note_reads` gives every compiler of that check all the readers. `jailsp` calls it in `semantic/compile.rs` and `jailint` in `driver/lint_root.rs`. A file several programs load is linted by one of them, so each `jailint` root also returns the notes it saw read (`RootResult::read_notes`), and the driver drops `unused_note` findings that any root read.
6. **Ask.** `ide_note_readers_at(file, offset)` lists the readers of the note at a position (`Analysis::definition` adds them). `ide_note_readers_in(file)` lists the note spans read in a file (the lint uses it).

```jai
for note: header.notes {
    if note.text == "Traced" ...                        // reads @Traced only
    if note.text.count == 5 && note.text[0] == #char "S" ... // reads 5-byte notes starting with S
}
```

## Precision

Reads are per note, not per `notes` array. Walking an array and loading `note.text` (the string view) reads no note. Only the text bytes count, so the usual `==` test reads only the notes it matches. `==` compares lengths before calling `memcmp`, and a `memcmp` that finds a difference is not a read. A prefix test (`begins_with`) or parsing `@Tag(args)` byte by byte reads the notes it looks at.

Not seen:

- Bytes read by foreign code, such as a C `strcmp` in the sandbox.
- Notes on code given to the program as a string (`add_build_string`). Such a note has no file.

## How to change it

- New ways for compile-time code to reach notes must give each note its own bytes and register them with a `NotePlace` while `Interp::note_watch` is on. Batch compiles keep sharing equal note strings in type info; only editor compiles pay for one global per note.
- A new memory-reading op in `interp/code` needs a `watch!` in `run_ops`, or the reads it does go unseen.
- The lint skips the compiler's own notes (`jaic::sema::COMPILER_NOTES`), jailint's [declaration notes](../tools/jailint.md#declaration-notes) (`jailint::LINT_NOTES`, kept equal to the documented tables by a test) and the notes other tools read (`jailint::TOOL_NOTES`: `jacket test`'s and `@native`). Add a note to the first when the compiler starts reading it, to the tables when a rule does, and to `TOOL_NOTES` when `jacket test` (`test/options.rs`) or another tool does; a jacket test checks the runner knows every name in it.

## Configuration

None. Recording is on whenever editor facts are (`IdeFacts`). The `unused_note` rule is in the `restriction` group, so it is off unless enabled:

```toml
[lint.rules]
unused_note = "warn"
```

## Dependencies

- `record_build.rs` (the only path that gives metaprograms `Code_Note`s), `sema/typeinfo/views.rs`, `interp/globals.rs`, `interp/code/run.rs`, `interp/intrinsics.rs`, `interp/ir_fallback.rs`.
- [IDE facts](ide-facts.md), [language server](language-server.md).
- Readers kept per analysis: 16384 (`MAX_READERS`).
