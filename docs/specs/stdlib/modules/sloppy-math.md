# Sloppy_Math


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `is_approximately_zero :: (f: float, epsilon := DEFAULT_EPSILON) -> bool`
- `values_are_close :: (f0: float, f1: float, epsilon := DEFAULT_EPSILON) -> bool`
- `values_are_close :: (a: Math.Vector2, b: Math.Vector2, epsilon := DEFAULT_EPSILON) -> bool`
- `values_are_close :: (a: Math.Vector3, b: Math.Vector3, epsilon := DEFAULT_EPSILON) -> bool`
- `max_of_type :: inline ($T: Type) -> T`
- `min_of_type :: inline ($T: Type) -> T`


## Behavioral contract

Provides approximate-zero and approximate-equality checks for scalars and `Vector2`/`Vector3`, using the file-local `DEFAULT_EPSILON` value `0.0001` unless overridden. The epsilon comparison is inclusive and absolute, not scaled to operand magnitude. `min_of_type`/`max_of_type` return representable bounds for supported numeric types; for float types, the minimum is the positive minimum-normal value, not the most-negative finite value. The epsilon constant is not a module export. The module depends on `Math` vector types and numeric type metadata.
