// Readers for executable, object and library files: ELF, Mach-O, PE/COFF, Apple universal binaries,
// `ar` archives and text-based dylib stubs (`.tbd`), plus the format constants and header structs.
//
// Each format has an `is_` check on the file's bytes, a `parse_` procedure for bytes already in
// memory and a `parse_..._file` procedure that reads a path first, such as [is_elf], [parse_elf] and
// [parse_elf_file]. The parse procedures return false on malformed input. Except for `.tbd`, the
// parsed result points into the bytes it was given (its `content`), so keep them alive while it is
// in use; on success, the bytes read by a `_file` procedure stay allocated for the result.

#load "../internal/executable/abi.jai";
#load "../internal/executable/bitfields.jai";
#load "../internal/executable/macho.jai";
#load "../internal/executable/coff.jai";
#load "../internal/executable/codeview.jai";
#load "../internal/executable/elf.jai";
#load "../internal/executable/universal.jai";
#load "../internal/executable/relocations.jai";
#load "../internal/executable/lexer.jai";
#load "../internal/executable/tbd.jai";
#load "../internal/executable/archive.jai";
