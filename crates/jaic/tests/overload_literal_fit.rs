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
