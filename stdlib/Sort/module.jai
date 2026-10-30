// In-place sorting of arrays by a comparator or a key, and ready-made comparators.

// Sorts an array in place and returns the same view, so calls can be chained.
//
// There are two forms. `quick_sort(array, compare)` takes a three-way comparator, `compare(a, b)` returning a
// negative number when `a` goes first, zero when they tie and a positive number otherwise (any numeric
// return type will do). It is an introsort: quicksort that falls back to heap sort on bad input, so it is
// `O(n log n)` in the worst case, and it does not keep the order of equal elements. `quick_sort(array,
// key)` takes a procedure that gives each element a key compared with `<`; the key is computed once per
// element, and elements with equal keys do keep their order (it is stable). That form allocates a temporary
// copy with `context.allocator` and frees it before returning.
//
// A comparator should be consistent (a strict weak order). An inconsistent one gives an unspecified order,
// but the sort stays within the array and always finishes. Arrays of fewer than two elements are returned untouched.
//
// ```jai
// Person :: struct { name: string; age: int; }
// people := Person.[.{"Ann", 31}, .{"Bo", 25}, .{"Cy", 31}];
//
// by_age :: (a: Person, b: Person) -> int { return a.age - b.age; }
// quick_sort(people, by_age);
// assert(people[0].name == "Bo");
//
// age_of :: (p: Person) -> int { return p.age; }
// quick_sort(people, age_of);   // stable: Ann stays before Cy
// assert(people[1].name == "Ann" && people[2].name == "Cy");
//
// numbers := int.[3, 1, 2];
// sorted := quick_sort(numbers, (a: int, b: int) -> int { return a - b; });
// assert(sorted[0] == 1 && sorted[2] == 3 && sorted.data == numbers.data);
// ```
quick_sort :: Impl.quick_sort;
// A three-way comparator for `quick_sort` and `intro_sort`: `-1`, `0` or `1` for `a < b`, a tie and `a > b`.
//
// The order is total: every `NaN` goes after all numbers (infinities included) and ties only with other
// `NaN`s, so a sort puts NaNs last. `-0.0` and `0.0` tie.
//
// ```jai
// values := float.[2.5, -1, 9];
// quick_sort(values, compare_floats);
// assert(values[0] == -1 && values[2] == 9);
//
// #import "Math";
// with_nan := float.[FLOAT32_NAN, 3, -2];
// quick_sort(with_nan, compare_floats);
// assert(with_nan[0] == -2 && with_nan[1] == 3 && with_nan[2] != with_nan[2]);
// ```
compare_floats :: Impl.compare_floats;
// A three-way comparator for strings, `-1`, `0` or `1`, comparing bytes in order, so UTF-8 text sorts by code point and upper case comes before lower case. A string that is a prefix of another goes first.
//
// ```jai
// names := string.["pear", "Apple", "fig", "figs"];
// quick_sort(names, compare_strings);
// assert(names[0] == "Apple" && names[1] == "fig" && names[2] == "figs" && names[3] == "pear");
// ```
compare_strings :: Impl.compare_strings;

#scope_file
Impl :: #import "internal/sort";
