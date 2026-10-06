// Type_Info_Enum has no notes field: enum notes only reach metaprograms through Code_Enum.
// rules: note.10
Mode :: enum { A; B; }

main :: () {
    n := type_info(Mode).notes;
}
