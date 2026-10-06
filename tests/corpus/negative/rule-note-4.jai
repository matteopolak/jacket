// #program_export keeps a procedure as a root even when nothing calls it, so its body is checked
// (an uncalled procedure without it would not be).
// rules: note.4
exported :: () -> int #program_export {
    return no_such_name;
}

main :: () {}
