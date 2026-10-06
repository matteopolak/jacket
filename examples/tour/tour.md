# A tour of Jai

This workspace is a small program that walks through the
language, one topic per file. main.jai loads every file and
runs each stop in order, printing a heading before each one.
The compiler, the interpreter and the standard library all
run in your browser, as WebAssembly.

Edit any file and the program runs again. Break things on
purpose: the error messages are part of the tour.

## Map

- [main.jai](main.jai): The itinerary: a table of procedures, run in a loop.
- [basics/basics.jai](basics/basics.jai): Variables, constants, procedures, multiple return values, named and default arguments, overloading.
- [types/structs.jai](types/structs.jai): Structs, `using`, `#as`, operator overloading, unions.
- [types/enums.jai](types/enums.jai): Enums, `enum_flags`, `#complete` switches.
- [data/arrays.jai](data/arrays.jai): Fixed arrays, views, `[..]` arrays, `for` loops, `remove`.
- [data/strings.jai](data/strings.jai): Strings, `print` formatting, String_Builder, here-strings.
- [memory/memory.jai](memory/memory.jai): `defer`, New and free, temporary storage, the context, a custom allocator and logger.
- [generics/polymorphism.jai](generics/polymorphism.jai): `$T`, polymorphic structs, `/interface`, `#modify`, `$$` and `#bake_arguments`.
- [meta/compile_time.jai](meta/compile_time.jai): `#run`, `#if`, `#assert`, `#insert`, `#code`.
- [meta/macros.jai](meta/macros.jai): `#expand` macros, backticks, custom `for` loops.
- [meta/reflection.jai](meta/reflection.jai): Type_Info, `Any`, notes, and a JSON writer.
- [meta/metaprogram.jai](meta/metaprogram.jai): Driving the compiler from your own code.
- [finale/raymarch.jai](finale/raymarch.jai): A ray marcher that draws a 3D scene in text.

## Things to try

- In types/enums.jai, delete a `case` from `pack_for`.
  `#complete` turns the missing case into a compile error.
- In meta/compile_time.jai, add a planet to PLANET_DATA.
  The enum and the gravity table both grow, generated
  at compile time.
- In finale/raymarch.jai, move CAMERA or LIGHT_DIRECTION,
  or add a third sphere to `scene`.
- Add a file of your own, `#load` it from main.jai and add
  a stop to the table in `main`.
