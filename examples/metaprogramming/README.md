# Metaprogramming

The compiler runs your code: `#run`, `#if`, `#assert`, `#insert`, `#code`, `#expand` macros,
reflection through `Type_Info`, and a metaprogram that compiles another program.

## Map

- [main.jai](main.jai): Runs the four sections.
- [compile_time.jai](compile_time.jai): `#run`, `#if`, `#assert`, `#insert`, `#code`.
- [macros.jai](macros.jai): `#expand` macros, backticks, custom `for` loops.
- [reflection.jai](reflection.jai): Type_Info, `Any`, notes, and a JSON writer.
- [metaprogram.jai](metaprogram.jai): Driving the compiler from your own code.

## Things to try

- In compile_time.jai, add a planet to `PLANET_DATA`. The enum and the gravity table both grow, generated at compile time.
- In reflection.jai, add a field to `Save_Game` and see it appear in the JSON.
