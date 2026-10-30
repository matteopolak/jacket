// Property tests: `@property` on a procedure whose arguments are generated. The procedure runs
// on many random inputs; when one fails, the input is shrunk to a smaller one that still fails
// and reported with the seed. See docs/extensions/testing.md.

// A source of pseudo-random numbers (splitmix64). The same seed gives the same numbers on every
// OS and in every mode, which is what makes a failing run repeatable. Custom generators take
// one and draw from it.
Rng :: struct {
    state: u64;
    // How big the values of this trial may be, from 1 on the first trial to 100 on the last.
    size: s64 = 1;
}

// A generator started from `seed`.
//
// - seed: the starting state; one seed always gives the same numbers.
rng_seeded :: (seed: u64) -> Rng {
    return .{ state = seed };
}

// The next 64 random bits.
//
// - rng: the generator to draw from and advance.
rng_u64 :: (rng: *Rng) -> u64 {
    rng.state += 0x9e3779b97f4a7c15;
    z := rng.state;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9;
    z = (z ^ (z >> 27)) * 0x94d049bb133111eb;

    return z ^ (z >> 31);
}

// A number from 0 to `count - 1`; 0 when `count` is 0.
//
// - rng: the generator to draw from.
// - count: how many values to choose from.
rng_below :: (rng: *Rng, count: u64) -> u64 {
    if count == 0 return 0;

    return rng_u64(rng) % count;
}

// A number from `low` to `high`, both included.
//
// - rng: the generator to draw from.
// - low: the smallest result.
// - high: the largest result.
rng_range :: (rng: *Rng, low: s64, high: s64) -> s64 {
    if high <= low return low;

    span := cast,no_check(u64) (high - low) + 1;
    if span == 0 return cast,no_check(s64) rng_u64(rng);

    return low + cast,no_check(s64) rng_below(rng, span);
}

// True once in `one_in` calls on average.
//
// - rng: the generator to draw from.
// - one_in: the odds, 1 in this many; below 1 counts as 1.
rng_chance :: (rng: *Rng, one_in: s64) -> bool {
    return rng_below(rng, cast(u64) max(one_in, 1)) == 0;
}

// A number from 0 up to, not including, 1.
//
// - rng: the generator to draw from.
rng_unit :: (rng: *Rng) -> float64 {
    return cast(float64) (rng_u64(rng) >> 11) / 9007199254740992.0;
}

// A generator the property uses for every argument (or part of one) of the type it returns,
// in place of the built-in one. Named in a `@generate(NAME)` note; values it makes are not
// shrunk.
//
// ```jai
// even :: (rng: *Rng) -> s64 { return rng_range(rng, -500, 500) * 2; }
//
// doubles :: (n: s64) { expect_equal((n * 3) % 2, 0); } @test @property @generate(even)
// ```
Custom_Generator :: struct {
    info: *Type_Info;
    make: (rng: *Rng, destination: *void);
}

// Wraps `make` as a `Custom_Generator` for `run_property`.
//
// - $make: draws one value from the `Rng`; its return type is the type the generator serves.
generator :: ($make: (rng: *Rng) -> $T) -> Custom_Generator {
    return .{ type_info(T), (rng: *Rng, destination: *void) {
        (cast(*T) destination).* = make(rng);
    } };
}

// Checks the arguments of a property: skips the trial, without failing it, when `condition`
// does not hold. A property that returns a value writes `if !condition return true;`.
//
// - condition: the trial is skipped when this is false.
assume :: (condition: bool) #expand {
    if !condition `return;
}

#scope_module

// The values being tried, one per generated argument.
Property_Arg :: struct {
    info: *Type_Info;
    pointer: *void;
}

// While the trials run, checks that fail are counted and not reported.
capturing := false;
trial_failed := false;
// Whether a property runs: a failed `assert` in it fails the input and the run goes on.
in_property := false;

// The type of the `index`-th generated argument of the procedure type `B`.
property_argument :: (B: Type, index: s64) -> Type {
    info := cast(*Type_Info_Procedure) B;
    return cast(Type) info.argument_types[index + leading_test(B)];
}

leading_test :: (B: Type) -> s64 {
    info := cast(*Type_Info_Procedure) B;
    if info.argument_types.count > 0 && info.argument_types[0] == type_info(*Test) return 1;

    return 0;
}

// The code `run_property` inserts: the arguments as variables, and a procedure that calls the
// property with them.
property_source :: (B: Type) -> string {
    info := cast(*Type_Info_Procedure) B;
    leading := leading_test(B);
    values := info.argument_types.count - leading;
    returns_bool := info.return_types.count == 1 && info.return_types[0].type == .BOOL;

    if values == 0 return "#assert false \"@property: the procedure needs arguments to generate\";";
    if info.return_types.count > 1 || (info.return_types.count == 1 && !returns_bool) return "#assert false \"@property: the procedure returns nothing or a bool (false fails the property)\";";

    builder: String_Builder;
    builder.allocator = temp;

    for 0..values - 1 {
        print_to_builder(*builder, "A% :: #run property_argument(B, %);\na%: A%;\n", it, it, it, it);
    }

    print_to_builder(*builder, "slots: [%] Property_Arg;\n", values);

    for 0..values - 1 {
        print_to_builder(*builder, "slots[%] = .{ type_info(A%), *a% };\n", it, it, it);
    }

    append(*builder, "invoke :: (t: *Test, slots: [] Property_Arg) -> bool {\n");
    append(*builder, ifx returns_bool then "    return body(" else "    body(");

    if leading append(*builder, "t");

    for 0..values - 1 {
        if it > 0 || leading append(*builder, ", ");
        print_to_builder(*builder, "(cast(*A%) slots[%].pointer).*", it, it);
    }

    append(*builder, ");\n");
    if !returns_bool append(*builder, "    return true;\n");

    append(*builder, "}\ncheck_property(t, slots, invoke, count, generators);\n");

    return builder_to_string(*builder,, temp);
}

#scope_export

// What a `@property` test runs: `body` on `count` generated inputs. Call it by hand to give
// the arguments custom generators or another count.
//
// - t: the running test.
// - $body: the property; its arguments are generated.
// - count: how many trials to run.
// - generators: custom generators that replace the built-in ones for their types.
run_property :: (t: *Test, $body: $B, count: s64 = DEFAULT_TRIALS, generators: [] Custom_Generator = .[]) {
    #insert -> string {
        return property_source(B);
    }
}

DEFAULT_TRIALS :: 100;

#scope_module

integer_of_bits :: (value: s64) -> Integer {
    if value < 0 return .{ true, cast,no_check(u64) -(value + 1) + 1 };

    return .{ false, cast(u64) value };
}

// Stores `value` in an integer of `info`'s width.
write_integer :: (info: *Type_Info, destination: *void, value: Integer) {
    bits := value.magnitude;
    if value.negative bits = cast,no_check(u64) 0 - bits;

    if info.runtime_size == {
        case 1; (cast(*u8) destination).* = cast,no_check(u8) bits;
        case 2; (cast(*u16) destination).* = cast,no_check(u16) bits;
        case 4; (cast(*u32) destination).* = cast,no_check(u32) bits;
        case; (cast(*u64) destination).* = bits;
    }
}

// The largest value of an integer type, and the magnitude of its smallest.
largest_positive :: (info: *Type_Info_Integer) -> u64 {
    bits := cast(u64) info.runtime_size * 8;
    if !info.signed return ifx bits == 64 then 0xFFFFFFFFFFFFFFFF else ((cast(u64) 1) << bits) - 1;

    return ((cast(u64) 1) << (bits - 1)) - 1;
}

smallest_magnitude :: (info: *Type_Info_Integer) -> u64 {
    if !info.signed return 0;

    bits := cast(u64) info.runtime_size * 8;

    return (cast(u64) 1) << (bits - 1);
}

MAX_SIZE :: 100;
// Attempts at finding a smaller input, over all arguments.
MAX_SHRINK_ATTEMPTS :: 4000;

// How the arguments are made: with `rng`, as large as `size` allows, by `custom` where one is
// given for the type.
Maker :: struct {
    rng: *Rng;
    size: s64;
    custom: [] Custom_Generator;
}

check_property :: (t: *Test, slots: [] Property_Arg, invoke: (t: *Test, slots: [] Property_Arg) -> bool, count: s64, generators: [] Custom_Generator) {
    for slots {
        problem := unsupported(it.info);
        if problem {
            report_failure(running_location(), tprint("@property cannot generate an argument of type %: %", type_text(it.info), problem));
            return;
        }
    }

    rng := rng_seeded(run_seed ^ name_hash(t.name));

    in_property = true;
    defer in_property = false;

    for trial: 1..count {
        mark := get_temporary_storage_mark();

        size := 1 + (trial - 1) * (MAX_SIZE - 1) / max(count - 1, 1);

        rng.size = size;
        maker := Maker.{ *rng, size, generators };
        for slots make_value(*maker, it.info, it.pointer);

        if holds(t, invoke, slots) {
            set_temporary_storage_mark(mark);
            continue;
        }

        original := describe_arguments(t, slots);
        steps := shrink(t, invoke, slots, generators);

        report_counterexample(t, invoke, slots, original, trial, steps);

        return;
    }

}

name_hash :: (name: string) -> u64 {
    hash: u64 = 0xcbf29ce484222325;
    for name hash = (hash ^ it) * 0x100000001b3;

    return hash;
}

// Whether the property held for what is in `slots`.
holds :: (t: *Test, invoke: (t: *Test, slots: [] Property_Arg) -> bool, slots: [] Property_Arg) -> bool {
    capturing = true;
    trial_failed = false;
    result := call_property(t, invoke, slots);

    capturing = false;

    return result && !trial_failed;
}

// Calls the property with copies of the arguments, so a property that changes what it is given
// (sorting an array in place) leaves the input it is judged on as it was.
call_property :: (t: *Test, invoke: (t: *Test, slots: [] Property_Arg) -> bool, slots: [] Property_Arg) -> bool {
    fresh: [..] Property_Arg;
    fresh.allocator = temp;

    for slots {
        block := talloc(max(it.info.runtime_size, 1));
        deep_copy(it.info, it.pointer, block);
        array_add(*fresh, .{ it.info, block });
    }

    return invoke(t, fresh);
}

report_counterexample :: (t: *Test, invoke: (t: *Test, slots: [] Property_Arg) -> bool, slots: [] Property_Arg, original: string, trial: s64, steps: s64) {
    location := running_location();
    shown := describe_arguments(t, slots);
    summary: String_Builder;
    summary.allocator = temp;

    print_to_builder(*summary, "property failed on trial % (seed %)\ncounterexample: %", trial, run_seed, shown);
    if steps > 0 print_to_builder(*summary, "\nshrunk from:    % in % steps", original, steps);

    print_to_builder(*summary, "\nrerun: jacket test --seed % --exact '%'", run_seed, t.name);

    // Whether it failed by returning false rather than by a failed check.
    capturing = true;
    trial_failed = false;
    result := call_property(t, invoke, slots);

    capturing = false;

    if !result && !trial_failed append(*summary, "\nthe property returned false");

    report_failure(location, builder_to_string(*summary,, temp));

    // The failing run once more, so its checks report where they are written.
    call_property(t, invoke, slots);
}

// The call being tried, as `name(a, b)`.
describe_arguments :: (t: *Test, slots: [] Property_Arg) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "%(", t.name);

    for slots {
        if it_index > 0 append(*builder, ", ");
        append(*builder, value_text(Any.{ it.info, it.pointer }));
    }

    append(*builder, ")");

    return builder_to_string(*builder,, temp);
}

value_text :: (value: Any) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    write_inline(*builder, value);

    return builder_to_string(*builder,, temp);
}

// A value on one line, the way it would be written in code: `Point.{x=1, y=2}`, `[1, 2]`.
write_inline :: (builder: *String_Builder, value: Any) {
    inner := underlying(value);
    info := inner.type;

    if info.type == {
        case .STRING;
            append(builder, quoted((cast(*string) inner.value_pointer).*));
        case .STRUCT;
            print_type_to_builder(builder, info);
            append(builder, ".{");
            first := true;

            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                if !first append(builder, ", ");

                first = false;
                print_to_builder(builder, "%=", it.name);
                write_inline(builder, Any.{ it.type, inner.value_pointer + it.offset_in_bytes });
            }

            append(builder, "}");
        case .ARRAY;
            element_type, count, data := array_view(inner);

            append(builder, "[");

            for 0..count - 1 {
                if it > 0 append(builder, ", ");
                write_inline(builder, element(element_type, data, it));
            }

            append(builder, "]");
        case .POINTER;
            target := (cast(**void) inner.value_pointer).*;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if !target {
                append(builder, "null");
            } else if pointee && pointee.type != .VOID {
                append(builder, "*");
                write_inline(builder, Any.{ pointee, target });
            } else {
                append(builder, "<pointer>");
            }
        case;
            print_to_builder(builder, "%", inner);
    }
}

type_text :: (info: *Type_Info) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    print_type_to_builder(*builder, info);

    return builder_to_string(*builder,, temp);
}

// Why values of the type cannot be made, or nothing.
unsupported :: (info: *Type_Info) -> string {
    if info.type == {
        case .INTEGER; #through;
        case .FLOAT; #through;
        case .BOOL; #through;
        case .STRING; #through;
        case .ENUM;
            return "";
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;
            if !pointee || pointee.type == .VOID return "";

            return unsupported(pointee);
        case .ARRAY;
            return unsupported((cast(*Type_Info_Array) info).element_type);
        case .VARIANT;
            return unsupported((cast(*Type_Info_Variant) info).variant_of);
        case .STRUCT;
            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                problem := unsupported(it.type);
                if problem return problem;
            }

            return "";
    }

    return "give it a @generate procedure";
}

#scope_file

custom_for :: (maker: *Maker, info: *Type_Info) -> *Custom_Generator {
    for *maker.custom if it.info == info return it;

    return null;
}

// Fills `destination` (room for a value of `info`) with a random value.
make_value :: (maker: *Maker, info: *Type_Info, destination: *void) {
    custom := custom_for(maker, info);
    if custom {
        custom.make(maker.rng, destination);
        return;
    }

    rng := maker.rng;

    if info.type == {
        case .INTEGER;
            write_integer(info, destination, random_integer(maker, cast(*Type_Info_Integer) info));
        case .ENUM;
            enum_info := cast(*Type_Info_Enum) info;
            values := enum_info.values;

            if values.count {
                write_integer(enum_info.internal_type, destination, integer_of_bits(values[rng_below(rng, cast(u64) values.count)]));
            }
        case .BOOL;
            (cast(*bool) destination).* = rng_chance(rng, 2);
        case .FLOAT;
            make_float(maker, info, destination);
        case .STRING;
            (cast(*string) destination).* = random_string(maker);
        case .ARRAY;
            make_array(maker, cast(*Type_Info_Array) info, destination);
        case .STRUCT;
            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                make_value(maker, it.type, destination + it.offset_in_bytes);
            }
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;
            target := cast(**void) destination;

            if !pointee || pointee.type == .VOID || rng_chance(rng, 4) {
                target.* = null;
            } else {
                target.* = talloc(max(pointee.runtime_size, 1));
                make_value(maker, pointee, target.*);
            }
        case .VARIANT;
            make_value(maker, (cast(*Type_Info_Variant) info).variant_of, destination);
    }
}

// A random value of the integer type: mostly small, sometimes an edge, now and then any.
random_integer :: (maker: *Maker, info: *Type_Info_Integer) -> Integer {
    rng := maker.rng;
    largest := largest_positive(info);
    smallest := smallest_magnitude(info);

    pick := rng_below(rng, 20);

    if pick == 0 return .{ false, 0 };
    if pick == 1 return .{ false, 1 };
    if pick == 2 && info.signed return .{ true, 1 };
    if pick == 3 return .{ false, largest };
    if pick == 4 && info.signed return .{ true, smallest };

    magnitude: u64;

    if pick < 7 {
        // Anywhere in the type's range.
        magnitude = rng_u64(rng);
    } else {
        limit := cast(u64) (maker.size * maker.size);
        magnitude = rng_below(rng, limit + 1);
    }

    negative := info.signed && rng_chance(rng, 2);

    if negative {
        if magnitude > smallest magnitude %= smallest + 1;
    } else {
        if magnitude > largest magnitude %= largest + 1;
    }

    return .{ negative && magnitude != 0, magnitude };
}

make_float :: (maker: *Maker, info: *Type_Info, destination: *void) {
    rng := maker.rng;
    value: float64;
    pick := rng_below(rng, 10);

    if pick == 0 {
        value = 0;
    } else if pick == 1 {
        value = 1;
    } else if pick == 2 {
        value = -1;
    } else {
        scale := cast(float64) (maker.size * maker.size);
        value = (rng_unit(rng) * 2 - 1) * scale;
        // Whole numbers are the common case worth hitting.
        if rng_chance(rng, 3) value = cast(float64) cast(s64) value;
    }

    if info.runtime_size == 4 {
        (cast(*float32) destination).* = cast(float32) value;
    } else {
        (cast(*float64) destination).* = value;
    }
}

// Mostly printable ASCII, with some line breaks, tabs and multi-byte characters.
random_string :: (maker: *Maker) -> string {
    rng := maker.rng;
    length := cast(s64) rng_below(rng, cast(u64) maker.size + 1);
    builder: String_Builder;
    builder.allocator = temp;

    for 1..length {
        pick := rng_below(rng, 20);

        if pick == 0 {
            append(*builder, "\n");
        } else if pick == 1 {
            append(*builder, "\t");
        } else if pick == 2 {
            append(*builder, EXTRA_CHARACTERS[rng_below(rng, cast(u64) EXTRA_CHARACTERS.count)]);
        } else {
            append(*builder, cast(u8) rng_range(rng, 0x20, 0x7e));
        }
    }

    return builder_to_string(*builder,, temp);
}

EXTRA_CHARACTERS :: string.["\xc3\xa9", "\xe4\xb8\xad", "\xf0\x9f\x98\x80", "\xc3\x9f"];

make_array :: (maker: *Maker, info: *Type_Info_Array, destination: *void) {
    element_type := info.element_type;
    stride := element_type.runtime_size;
    count: s64;
    data: *void;

    if info.array_type == .FIXED {
        count = info.array_count;
        data = destination;
    } else {
        count = cast(s64) rng_below(maker.rng, cast(u64) maker.size + 1);
        data = talloc(max(count * stride, 1));
    }

    // Elements of an array are smaller than the array, so a list of lists stays small.
    inner := maker.*;
    inner.size = maker.size / 2 + 1;

    for 0..count - 1 make_value(*inner, element_type, data + it * stride);

    if info.array_type == .VIEW {
        view := cast(*[] u8) destination;
        view.* = .{ count, data };
    } else if info.array_type == .RESIZABLE {
        // A resizable array is its data, count, allocated and allocator, in that order.
        resizable := cast(*[..] u8) destination;
        resizable.count = count;
        resizable.data = data;
        resizable.allocated = count;
        resizable.allocator = temp;
    }
}
