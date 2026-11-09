#import "Basic";
#import "String";
#import "Extensions/Testing";
#import "Extensions/Vt";
#import,dir "support";
#load "../../test-support/counting-allocator.jai";

Digest :: struct {
    value: u64;
    actions: int;
}

digest_action :: (data: *void, a: Vt_Action) {
    d := cast(*Digest) data;
    d.actions += 1;
    d.value = (d.value << 1) ^ (d.value >> 63) ^ (cast(u64) a.kind) ^ (cast(u64) a.code);
    for a.params d.value = (d.value << 1) ^ it.value ^ (cast(u64) it.subparam);
    for a.intermediates d.value = (d.value << 1) ^ it;
    for a.osc for it d.value = (d.value << 1) ^ it;

    d.value = (d.value << 1) ^ a.private_marker;
}

no_allocations_and_random_chunking :: () {
    tally: Tally;
    whole, chunked: Digest;

    {
        push_allocator(tally_proc, *tally);
        bytes: [8192] u8;
        seed: u32 = 12345;

        for *bytes {
            seed = (seed << 7) ^ (seed >> 3) ^ 0x95a319c7;
            it.* = cast(u8) (seed & 255);
        }

        for c1: 0..1 {
            whole = .{};
            chunked = .{};
            p, q: Vt_Parser;
            p.options.c1 = c1 != 0;
            q.options = p.options;
            vt_feed(*p, cast(string) bytes, digest_action, *whole);
            for i: 0..bytes.count - 1 vt_feed(*q, string.{ 1, bytes.data + i }, digest_action, *chunked);

            vt_finish(*p, digest_action, *whole);
            vt_finish(*q, digest_action, *chunked);
            expect_equal(whole.value, chunked.value);
            expect_equal(whole.actions, chunked.actions);
        }
    }

    expect_equal(tally.outstanding, 0);
    expect_equal(tally.allocations, 0);
} @test

state_transitions_and_cancellation :: () {
    cases := string.[
        "\x1b[12\n;3X", "\x1b[1 $3mZ", "\x1bP1 $3qignored\x1b\\Z",
        "\x1bP1$q\x00\x7f\x07\x18Z", "\x1b]2;\x00title\x1aZ",
        "\x1b_ignored\x07\x1b\\Z", "\x1b^ignored\x1b\\Z", "\x1bXignored\x18Z",
        "\x1b[??3mZ", "\x1b[1\x7f2m", "\x1bP1\x00;2$qx\x1b\\",
        "\x90qdata\x9cZ", "\x9d2;title\x9cZ", "\x98ignored\x9cZ"
    ];
    for bytes: cases {
        whole := trace(bytes, 0, .{ c1 = true });

        defer free(whole);

        for split: 0..bytes.count {
            chunked := trace(bytes, split, .{ c1 = true });
            defer free(chunked);
            expect_equal(chunked, whole);
        }
    }

    actual := trace("\x1bP1$q\x00\x7f\x07\x18Z", 0);

    defer free(actual);
    expect_equal(actual, "DCS_HOOK:113/1:false[$]?0|DCS_PUT:0[]?0|DCS_PUT:7[]?0|DCS_UNHOOK:0[]?0|EXECUTE:24[]?0|PRINT:90[]?0|");
    invalid := trace("\x1b[1 $3mZ", 0);

    defer free(invalid);
    expect_equal(invalid, "PRINT:90[]?0|");
} @test

invalid_options_and_limits :: () {
    bad := Vt_Options.[.{ max_params = 0 }, .{ max_params = 65 }, .{ max_intermediates = -1 }, .{ max_intermediates = 9 }, .{ max_osc_bytes = -1 }, .{ max_osc_bytes = 4097 }];
    for options: bad {
        p: Vt_Parser;
        p.options = options;
        d: Digest;
        error := vt_feed(*p, "abc", digest_action, *d);

        expect_equal(error.kind, Vt_Error_Kind.OPTIONS);
        expect_equal(p.offset, cast(u64) 0);
        expect_equal(d.actions, 0);
    }

    p: Vt_Parser;
    p.options.max_intermediates = 1;
    d: Digest;
    error := vt_feed(*p, "\x1b !A\x1b[1 !m\x1bP1 !qignored\x1b\\Z", digest_action, *d);

    expect_equal(error.kind, Vt_Error_Kind.LIMIT);
    expect_equal(d.actions, 2); // ST escape dispatch and Z; all overflowed commands dropped
    expect_equal(p.state, Vt_State.GROUND);
    p.options.max_param_value = 4294967295;
    c: Capture;

    vt_feed(*p, "\x1b[999999999999999999999999999m", record, *c);
    text := builder_to_string(*c.text);

    defer free(text);
    expect_equal(text, "CSI_DISPATCH:109/4294967295:false[]?0|");
} @test

hard_capacity_boundaries_and_finish :: () {
    p: Vt_Parser;
    p.options.max_params = 64;
    p.options.max_intermediates = 8;
    d: Digest;

    vt_feed(*p, "\x1b]", digest_action, *d);
    payload: [4096] u8;
    for *payload it.* = #char "x";

    error := vt_feed(*p, cast(string) payload, digest_action, *d);

    expect_equal(error.kind, Vt_Error_Kind.NONE);
    error = vt_feed(*p, "\x07", digest_action, *d);
    expect_equal(error.kind, Vt_Error_Kind.NONE);
    expect_equal(d.actions, 1);
    vt_feed(*p, "\x1b]", digest_action, *d);
    vt_feed(*p, cast(string) payload, digest_action, *d);
    error = vt_feed(*p, "x\x07", digest_action, *d);
    expect_equal(error.kind, Vt_Error_Kind.LIMIT);
    expect_equal(d.actions, 1);
    vt_feed(*p, "\x1b[", digest_action, *d);
    for 1..63 vt_feed(*p, ";", digest_action, *d);

    error = vt_feed(*p, "m\x1b !\"#$%&'A", digest_action, *d);
    expect_equal(error.kind, Vt_Error_Kind.NONE);
    expect_equal(d.actions, 3);
    vt_feed(*p, "\x1bPqabc", digest_action, *d);
    before := d.actions;
    offset := p.offset;

    vt_finish(*p, digest_action, *d);
    expect_equal(d.actions, before + 1);
    expect_equal(p.state, Vt_State.GROUND);
    expect_equal(p.offset, offset);
    expect_equal(p.options.max_params, 64);
    error = vt_feed(*p, "abc", null);
    expect_equal(error.kind, Vt_Error_Kind.OPTIONS);
    expect_equal(p.offset, offset);
} @test
