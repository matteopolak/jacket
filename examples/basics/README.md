# Basics

Declarations and procedures, structs and unions, enums and flags. [main.jai](main.jai) runs one
section per file, in order.

Edit any file and the program runs again. Hover any name to see its type and documentation:
comments directly above a declaration are its docs, in Markdown, and `[name]` links in them jump
to what they name (try `divide` and `hail` in basics.jai).

## Map

- [basics.jai](basics.jai): Variables, constants, procedures, multiple return values, named and default arguments, overloading, doc comments.
- [structs.jai](structs.jai): Structs, `using`, `#as`, operator overloading, unions.
- [enums.jai](enums.jai): Enums, `enum_flags`, `#complete` switches.

## Things to try

- In enums.jai, delete a `case` from `pack_for`. `#complete` turns the missing case into a compile error.
- In structs.jai, add a field to `Creature` and watch `Knight` and `Dragon` pick it up through `using`.
