//! Overloads are ranked by how well a literal argument fits each candidate, and a call that two
//! candidates take equally well is an error.
mod common;
use common::compile_source;

fn outcome(source: &str) -> Result<(), String> {
    let (c, result) = compile_source(source);
    result.map_err(|error| error.render(&c.src.sources))
}

fn accepts(source: &str) {
    if let Err(message) = outcome(source) {
        panic!("{source}: {message}");
    }
}

fn ambiguous(source: &str) {
    let message = outcome(source).expect_err(source);
    assert!(
        message.contains("Procedure call matches")
            && message.contains("is ambiguous for these arguments"),
        "{source}: {message}"
    );
}

const FIELDS: &str = "
V :: struct { a: s32; }
W :: struct { a: s32; c: s32; }
take :: (p: V) -> s32 { return 1; }
take :: (p: W) -> s32 { return 2; }
";

#[test]
fn a_named_field_a_candidate_lacks_rules_it_out() {
    accepts(&format!(
        "{FIELDS} one :: #run take(.{{c = 1}}); two :: #run take(.{{a = 1}}); #assert(one == 2);"
    ));
}

const SHAPES: &str = "
V :: struct { a: s32; b: s32; }
W :: struct { a: s32; b: s32; c: s32 = 9; }
take :: (p: V) -> s32 { return 1; }
take :: (p: W) -> s32 { return 2; }
";

#[test]
fn the_candidate_the_values_fill_best_wins() {
    accepts(&format!(
        "{SHAPES}
        pair :: #run take(.{{1, 2}});
        triple :: #run take(.{{1, 2, 3}});
        typed :: #run take(V.{{1, 2}});
        #assert(pair == 1 && triple == 2 && typed == 1);"
    ));
}

#[test]
fn a_literal_that_says_nothing_about_its_shape_is_ambiguous() {
    ambiguous(&format!("{SHAPES} main :: () {{ take(.{{}}); }}"));
}

#[test]
fn null_and_array_literals_need_a_single_best_overload() {
    ambiguous("take :: (p: *s32) {} take :: (p: *u8) {} main :: () { take(null); }");
    ambiguous("take :: (p: [2] s32) {} take :: (p: [] s32) {} main :: () { take(.[1, 2]); }");
    // A typed argument matching one overload exactly is not ambiguous.
    accepts(
        "take :: (p: *s32) {} take :: (p: *u8) {} main :: () { x: s32; take(*x); take(null_u8()); } null_u8 :: () -> *u8 { return null; }",
    );
}

#[test]
fn values_beyond_the_positional_members_rule_a_candidate_out() {
    // `Vec3` has overlays, which take no value of their own.
    accepts(
        "Vec2 :: struct { x, y: float; }
        Vec3 :: struct { x, y, z: float; #overlay(x) xy: Vec2; #overlay(y) yz: Vec2; }
        Vec4 :: struct { x, y, z, w: float; #overlay(x) xyz: Vec3; #overlay(y) yzw: Vec3; #overlay(z) zw: Vec2; }
        pick :: (x: float) -> s32 { return 1; }
        pick :: (c: Vec3) -> s32 { return 3; }
        pick :: (c: Vec4) -> s32 { return 4; }
        four :: #run pick(.{1, 2, 3, 4});
        three :: #run pick(.{1, 2, 3});
        #assert(four == 4 && three == 3);",
    );
}

#[test]
fn a_value_that_cannot_be_stored_in_a_member_rules_a_candidate_out() {
    accepts(
        "Number :: struct { value: u8; extra: int = 6; }
        Flag :: struct { value: bool; }
        choose :: (item: Number) -> s32 { return 1; }
        choose :: (item: Flag) -> s32 { return 2; }
        number :: #run choose(.{ value = 7 });
        flag :: #run choose(.{ value = true });
        positional :: #run choose(.{ true });
        #assert(number == 1 && flag == 2 && positional == 2);",
    );
}

#[test]
fn a_typed_literal_and_a_code_argument_are_not_ambiguous() {
    accepts(
        "V :: struct { a: s32; }
        element :: (style := V.{}, body: Code = #code {}) #expand {}
        element :: (body: Code) #expand {}
        element :: (style: V, key: string) #expand {}
        main :: () { element(V.{ a = 1 }, #code { x := 1; }); }",
    );
}

#[test]
fn overloads_of_an_outer_scope_do_not_rival_a_nearer_one() {
    accepts(
        "E :: enum { A; B; }
        take :: (e: E) -> s32 { return 1; }
        main :: () {
            F :: enum { A; B; }
            take :: (f: F) -> s32 { return 2; }
            x := take(.A);
            #assert(type_of(x) == s32);
        }",
    );
}

#[test]
fn a_candidate_taking_the_argument_itself_beats_one_gathering_it_into_a_variadic() {
    accepts(
        "V :: struct { a: s32; }
        add :: (item: V) -> s32 { return 1; }
        add :: (items: ..V) -> s32 { return 2; }
        one :: #run add(.{ a = 1 });
        #assert(one == 1);",
    );
}

#[test]
fn polymorphic_overloads_that_fit_equally_are_ambiguous() {
    let arguments = [
        "5",
        "5.5",
        "\"text\"",
        "true",
        "V.{1, 2}",
        "s32.[1, 2, 3]",
        "null",
        "cast(u8) 3",
    ];
    for argument in arguments {
        ambiguous(&format!(
            "V :: struct {{ a: s32; b: s32; }}
            f :: (x: $T) -> s32 {{ return 1; }}
            f :: (x: $U) -> s32 {{ return 2; }}
            main :: () {{ f({argument}); }}"
        ));
    }
    // A header that pins down more of the type is not a tie.
    accepts(
        "f :: (x: *$T) -> s32 { return 1; }
        f :: (x: $U) -> s32 { return 2; }
        main :: () { v: s32; #assert(type_of(f(*v)) == s32); }",
    );
}

#[test]
fn polymorphic_overloads_of_another_shape_are_not_rivals() {
    // An empty variadic and a parameter list without one differ in shape.
    accepts(
        "add :: (array: *[..] $T) -> s32 { return 1; }
        add :: (array: *[..] $T, items: ..T) -> s32 { return 2; }
        main :: () { a: [..] s32; x := add(*a); #assert(type_of(x) == s32); }",
    );
    // A header with fewer polymorphic variables is the closer reading.
    accepts(
        "read :: (d: [] u8) -> s32 { return 1; }
        read :: (d: [] $T) -> s32 { return 2; }
        main :: () { d: [4] u8; x := read(d); #assert(type_of(x) == s32); }",
    );
}

#[test]
fn an_untyped_number_fits_every_numeric_parameter_alike() {
    accepts(
        "take :: (p: u8) -> s32 { return 1; }
        take :: (p: s32) -> s32 { return 2; }
        one :: #run take(200);
        #assert(one == 1);",
    );
}

#[test]
fn a_named_literal_leaves_members_with_initializers_to_them() {
    ambiguous(&format!(
        "{SHAPES} main :: () {{ take(.{{ a = 1, b = 2 }}); }}"
    ));
    // A member with no initializer is a shortfall, so the type without it is closer.
    accepts(&format!(
        "{FIELDS} one :: #run take(.{{ a = 1 }}); #assert(one == 1);"
    ));
}

#[test]
fn an_element_parameter_beats_a_constant_parameter_with_a_default() {
    accepts(
        "push :: (arr: *[..] $T, $flag := true) -> s32 { return 1; }
        push :: (arr: *[..] $T, elem: T) -> u8 { return 2; }
        main :: () {
            words: [..] string;
            a := push(*words, \"text\");
            #assert(type_of(a) == u8);
            number: s32;
            numbers: [..] s32;
            b := push(*numbers, number);
            #assert(type_of(b) == u8);
        }",
    );
}

#[test]
fn concrete_elements_rank_above_binders_in_implicit_generic_headers() {
    accepts(
        "Reader :: struct (K:Type) {value:K;}
        read :: (r:*Reader, data:[]u8) -> int {return 1;}
        read :: (r:*Reader, data:[]$T) -> int {return 2;}
        check :: () -> bool {
            r:Reader(s64); bytes:[]u8; words:[]u32;
            return read(*r,bytes)==1 && read(*r,words)==2;
        }
        #assert check();",
    );
}
