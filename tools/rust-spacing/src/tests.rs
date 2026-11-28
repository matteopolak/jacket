use super::space;

/// Asserts that `input` becomes `expected`, and that `expected` is a fixed point.
#[track_caller]
fn check(input: &str, expected: &str) {
    let out = space(input).unwrap();

    assert_eq!(out, expected, "first pass");
    assert_eq!(space(&out).unwrap(), expected, "second pass is not a no-op");
}

#[track_caller]
fn unchanged(input: &str) {
    check(input, input);
}

#[test]
fn guard_gets_a_blank_line_on_both_sides() {
    check(
        "fn f() {\n    a();\n    if x {\n        b();\n    }\n    c();\n}\n",
        "fn f() {\n    a();\n\n    if x {\n        b();\n    }\n\n    c();\n}\n",
    );
}

#[test]
fn every_guard_kind() {
    let heads = [
        ("if x {", "b();"),
        ("match x {", "_ => b(),"),
        ("loop {", "b();"),
        ("while x {", "b();"),
        ("for i in 0..3 {", "b();"),
        ("'a: loop {", "b();"),
    ];

    for (head, body) in heads {
        let input =
            format!("fn f() {{\n    a();\n    {head}\n        {body}\n    }}\n    c();\n}}\n");
        let expected =
            format!("fn f() {{\n    a();\n\n    {head}\n        {body}\n    }}\n\n    c();\n}}\n");

        check(&input, &expected);
    }
}

#[test]
fn guard_that_opens_or_closes_its_block_needs_nothing() {
    unchanged("fn f() {\n    if x {\n        b();\n    }\n}\n");
    unchanged("fn f() {\n    a();\n\n    if x {\n        b();\n    }\n}\n");
}

#[test]
fn one_line_guard_is_an_ordinary_statement() {
    unchanged("fn f() {\n    a();\n    if x { b(); }\n    c();\n}\n");
}

#[test]
fn else_chain_is_one_guard() {
    check(
        "fn f() {\n    a();\n    if x {\n        b();\n    } else if y {\n        c();\n    } else {\n        d();\n    }\n    e();\n}\n",
        "fn f() {\n    a();\n\n    if x {\n        b();\n    } else if y {\n        c();\n    } else {\n        d();\n    }\n\n    e();\n}\n",
    );
}

#[test]
fn comment_above_a_guard_stays_attached() {
    check(
        "fn f() {\n    a();\n    // Skip the empty case.\n    // Really.\n    if x {\n        b();\n    }\n}\n",
        "fn f() {\n    a();\n\n    // Skip the empty case.\n    // Really.\n    if x {\n        b();\n    }\n}\n",
    );
    check(
        "fn f() {\n    a();\n    /* note */\n    if x {\n        b();\n    }\n}\n",
        "fn f() {\n    a();\n\n    /* note */\n    if x {\n        b();\n    }\n}\n",
    );
    check(
        "fn f() {\n    a();\n    /*\n     * note\n     */\n    if x {\n        b();\n    }\n}\n",
        "fn f() {\n    a();\n\n    /*\n     * note\n     */\n    if x {\n        b();\n    }\n}\n",
    );
}

#[test]
fn detached_comment_is_not_moved() {
    unchanged("fn f() {\n    a();\n\n    // note\n\n    if x {\n        b();\n    }\n}\n");
}

#[test]
fn trailing_comment_on_the_previous_line_is_code_not_a_header() {
    check(
        "fn f() {\n    a(); // note\n    if x {\n        b();\n    }\n}\n",
        "fn f() {\n    a(); // note\n\n    if x {\n        b();\n    }\n}\n",
    );
}

#[test]
fn attribute_stays_with_its_statement() {
    check(
        "fn f() {\n    a();\n    #[allow(unused)]\n    let x = 1;\n    if y {\n        b();\n    }\n}\n",
        "fn f() {\n    a();\n    #[allow(unused)]\n    let x = 1;\n\n    if y {\n        b();\n    }\n}\n",
    );
}

#[test]
fn let_run_is_set_apart_from_the_next_statement() {
    check(
        "fn f() {\n    let a = 1;\n    let b = 2;\n    run(a, b);\n    done();\n}\n",
        "fn f() {\n    let a = 1;\n    let b = 2;\n\n    run(a, b);\n    done();\n}\n",
    );
}

#[test]
fn multi_line_let_is_still_a_let() {
    check(
        "fn f() {\n    let a = g(\n        1,\n    );\n    let b = 2;\n    run(a, b);\n}\n",
        "fn f() {\n    let a = g(\n        1,\n    );\n    let b = 2;\n\n    run(a, b);\n}\n",
    );
}

#[test]
fn lone_let_stays_with_a_short_last_statement() {
    unchanged("fn f() {\n    let n = g();\n    n + 1\n}\n");
    unchanged("fn f() {\n    let n = g();\n    return n;\n}\n");
}

#[test]
fn a_run_of_lets_is_set_apart_even_from_a_short_last_statement() {
    check(
        "fn f() {\n    let a = 1;\n    let b = 2;\n    a + b\n}\n",
        "fn f() {\n    let a = 1;\n    let b = 2;\n\n    a + b\n}\n",
    );
}

#[test]
fn return_ending_a_longer_block_is_set_apart() {
    check(
        "fn f() {\n    a();\n    b();\n    return 1;\n}\n",
        "fn f() {\n    a();\n    b();\n\n    return 1;\n}\n",
    );
    check(
        "fn f() {\n    loop {\n        a();\n        b();\n        break;\n    }\n}\n",
        "fn f() {\n    loop {\n        a();\n        b();\n\n        break;\n    }\n}\n",
    );
    check(
        "fn f() {\n    for x in y {\n        a(x);\n        b(x);\n        continue;\n    }\n}\n",
        "fn f() {\n    for x in y {\n        a(x);\n        b(x);\n\n        continue;\n    }\n}\n",
    );
}

#[test]
fn return_ending_a_two_statement_block_stays_close() {
    unchanged("fn f() {\n    a();\n    return 1;\n}\n");
}

#[test]
fn return_in_the_middle_is_left_alone() {
    unchanged("fn f() {\n    a();\n    b();\n    return 1;\n    c();\n}\n");
}

#[test]
fn nested_blocks_are_all_visited() {
    check(
        "fn f() {\n    let g = || {\n        a();\n        if x {\n            b();\n        }\n    };\n}\n",
        "fn f() {\n    let g = || {\n        a();\n\n        if x {\n            b();\n        }\n    };\n}\n",
    );
}

#[test]
fn item_inside_a_block_is_set_apart_when_multi_line() {
    check(
        "fn f() {\n    a();\n    fn g() {\n        b();\n    }\n    g();\n}\n",
        "fn f() {\n    a();\n\n    fn g() {\n        b();\n    }\n\n    g();\n}\n",
    );
}

#[test]
fn multi_line_items_are_separated() {
    check("fn a() {\n}\nfn b() {\n}\n", "fn a() {\n}\n\nfn b() {\n}\n");
    check(
        "struct A {\n    x: u8,\n}\nimpl A {\n    fn a(&self) {\n    }\n    fn b(&self) {\n    }\n}\n",
        "struct A {\n    x: u8,\n}\n\nimpl A {\n    fn a(&self) {\n    }\n\n    fn b(&self) {\n    }\n}\n",
    );
}

#[test]
fn one_line_items_may_stay_together() {
    unchanged("const A: u8 = 1;\nconst B: u8 = 2;\ntype T = u8;\n");
    unchanged("trait T {\n    fn a(&self);\n    fn b(&self);\n}\n");
}

#[test]
fn attribute_makes_an_item_multi_line() {
    check(
        "const A: u8 = 1;\n#[derive(Debug)]\nstruct B;\n",
        "const A: u8 = 1;\n\n#[derive(Debug)]\nstruct B;\n",
    );
}

#[test]
fn comment_above_an_item_stays_attached() {
    check(
        "fn a() {\n}\n/// Docs.\nfn b() {\n}\n",
        "fn a() {\n}\n\n/// Docs.\nfn b() {\n}\n",
    );
}

#[test]
fn imports_are_set_apart_from_what_follows() {
    check(
        "use a::B;\nuse c::D;\nmod e;\nconst X: u8 = 1;\n",
        "use a::B;\nuse c::D;\nmod e;\n\nconst X: u8 = 1;\n",
    );
    unchanged("use a::B;\n\nuse c::D;\n\nconst X: u8 = 1;\n");
}

#[test]
fn consecutive_imports_are_left_to_rustfmt() {
    unchanged("use a::B;\nuse c::{\n    D,\n    E,\n};\nuse f::G;\n");
    unchanged("mod a;\n#[cfg(test)]\nmod tests;\nuse b::C;\n");
}

#[test]
fn documented_consts_stay_together() {
    unchanged(
        "impl A {\n    /// One.\n    pub const X: u8 = 1;\n    /// Two.\n    pub const Y: u8 = 2;\n}\n",
    );
    unchanged("/// One.\nconst X: u8 = 1;\n/// Two.\n#[cfg(unix)]\ntype T = u8;\n");
}

#[test]
fn imports_in_a_block_are_left_to_rustfmt() {
    unchanged(
        "fn f() {\n    use a::{\n        B,\n        C,\n    };\n    use d::E;\n\n    run();\n}\n",
    );
}

#[test]
fn inner_attributes_are_left_before_the_first_item() {
    unchanged("#![allow(unsafe_code)]\n\nuse a::B;\nuse c::D;\n");
    unchanged(
        "#![expect(\n    clippy::too_many_arguments,\n    reason = \"mirrors the C API\"\n)]\n\nuse a::B;\n",
    );
}

#[test]
fn nested_modules_are_visited() {
    check(
        "mod m {\n    fn a() {\n    }\n    fn b() {\n    }\n}\n",
        "mod m {\n    fn a() {\n    }\n\n    fn b() {\n    }\n}\n",
    );
}

#[test]
fn statements_sharing_a_line_are_not_split() {
    unchanged("fn f() {\n    a(); if x {\n        b();\n    }\n}\n");
}

#[test]
fn macro_bodies_are_not_touched() {
    unchanged(
        "fn f() {\n    m! {\n        a();\n        if x {\n            b();\n        }\n        c();\n    }\n}\n",
    );
}

#[test]
fn multi_line_string_is_never_split() {
    unchanged("fn f() {\n    let s = \"a\n    if x {\n    b\";\n}\n");
}

#[test]
fn existing_blank_lines_are_kept() {
    unchanged("fn f() {\n    a();\n\n\n    if x {\n        b();\n    }\n}\n");
}

#[test]
fn not_rust_is_an_error() {
    assert!(space("fn (").is_err());
}

#[test]
fn text_without_a_final_newline_round_trips() {
    check(
        "fn f() {\n    a();\n    if x {\n        b();\n    }\n}",
        "fn f() {\n    a();\n\n    if x {\n        b();\n    }\n}",
    );
}
