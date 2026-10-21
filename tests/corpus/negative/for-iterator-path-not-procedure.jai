// A dotted `for:` iterator must name a procedure, not a value.
Holder :: struct { n: int; }

main :: () {
    holder: Holder;
    values: [3] int;
    for: holder.n values { }
}
