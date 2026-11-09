// Expected behavior derived from vte parser tests; no implementation code is copied.
// Pin: 89c9c22f5b8fcc3feba0a9d9f50a020801e74acc (tools/vt_conformance/fetch.py).
// Copyright (c) 2016 Joe Wilm
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
// Adaptations: private markers are separate, overflow drops the sequence with LIMIT,
// C1 is opt-in, fixed-buffer constructor cases use options, and finish closes DCS.
// No cases skipped. Each row is checked at every two-chunk split.
#import "Basic";
#import "String";
#import "Extensions/Testing";
#import "Extensions/Vt";
#import,dir "support";

Conformance_Case :: struct {
    name: string;
    input: string;
    expected: string;
    options: Vt_Options;
}

conformance_cases :: () {
    cases := Conformance_Case.[
        .{ "parse_osc", "\x1b]2;title\x07", "OSC_DISPATCH:0[]?0{2}{title}|", .{} },
        .{ "parse_empty_osc", "\x1b]\x07", "OSC_DISPATCH:0[]?0{}|", .{} },
        .{ "parse_osc_max_params", "\x1b];;;\x1b", "", .{ max_params = 3 } },
        .{ "osc_bell_terminated", "\x1b]11;ff/00/ff\x07", "OSC_DISPATCH:0[]?0{11}{ff/00/ff}|", .{} },
        .{ "osc_c0_st_terminated", "\x1b]11;ff/00/ff\x1b\\", "OSC_DISPATCH:0[]?0{11}{ff/00/ff}|ESC_DISPATCH:92[]?0|", .{} },
        .{ "parse_osc_with_utf8_arguments", "\x0d\x1b]2;echo \xc2\xaf\\_(\xe3\x83\x84)_/\xc2\xaf\x07", "EXECUTE:13[]?0|OSC_DISPATCH:0[]?0{2}{echo \xc2\xaf\\_(\xe3\x83\x84)_/\xc2\xaf}|", .{} },
        .{ "osc_containing_string_terminator", "\x1b]2;\xe6\x9c\xab\x1b\\", "OSC_DISPATCH:0[]?0{2}{\xe6\x9c\xab}|ESC_DISPATCH:92[]?0|", .{} },
        .{ "exceed_max_buffer_size", "\x1b]52;aaaaaaaaaaaaaaaaaa\x07", "", .{ max_osc_bytes = 8 } },
        .{ "parse_csi_max_params", "\x1b[1;1;p", "CSI_DISPATCH:112/1:false/1:false/0:false[]?0|", .{ max_params = 3 } },
        .{ "parse_csi_params_ignore_long_params", "\x1b[1;1;1;p", "", .{ max_params = 3 } },
        .{ "parse_csi_params_trailing_semicolon", "\x1b[4;m", "CSI_DISPATCH:109/4:false/0:false[]?0|", .{} },
        .{ "parse_csi_params_leading_semicolon", "\x1b[;4m", "CSI_DISPATCH:109/0:false/4:false[]?0|", .{} },
        .{ "parse_long_csi_param", "\x1b[9223372036854775808m", "CSI_DISPATCH:109/65535:false[]?0|", .{} },
        .{ "csi_reset", "\x1b[3;1\x1b[?1049h", "CSI_DISPATCH:104/1049:false[]?63|", .{} },
        .{ "csi_subparameters", "\x1b[38:2:255:0:255;1m", "CSI_DISPATCH:109/38:false/2:true/255:true/0:true/255:true/1:false[]?0|", .{} },
        .{ "parse_dcs_max_params", "\x1bP1;1;1;p", "", .{ max_params = 3 } },
        .{ "dcs_reset", "\x1b[3;1\x1bP1$tx\x9c", "DCS_HOOK:116/1:false[$]?0|DCS_PUT:120[]?0|DCS_UNHOOK:0[]?0|", .{ c1 = true } },
        .{ "parse_dcs", "\x1bP0;1|17/ab\x9c", "DCS_HOOK:124/0:false/1:false[]?0|DCS_PUT:49[]?0|DCS_PUT:55[]?0|DCS_PUT:47[]?0|DCS_PUT:97[]?0|DCS_PUT:98[]?0|DCS_UNHOOK:0[]?0|", .{ c1 = true } },
        .{ "intermediate_reset_on_dcs_exit", "\x1bP=1sZZZ\x1b+\\", "DCS_HOOK:115/1:false[]?61|DCS_PUT:90[]?0|DCS_PUT:90[]?0|DCS_PUT:90[]?0|DCS_UNHOOK:0[]?0|ESC_DISPATCH:92[+]?0|", .{} },
        .{ "esc_reset", "\x1b[3;1\x1b(A", "ESC_DISPATCH:65[(]?0|", .{} },
        .{ "esc_reset_intermediates", "\x1b[?2004l\x1b#8", "CSI_DISPATCH:108/2004:false[]?63|ESC_DISPATCH:56[#]?0|", .{} },
        .{ "params_buffer_filled_with_subparam", "\x1b[::::x\x1b", "", .{ max_params = 4 } },
        .{ "build_with_fixed_size", "\x1b]2;ok\x07", "OSC_DISPATCH:0[]?0{2}{ok}|", .{ max_osc_bytes = 4 } },
        .{ "exceed_fixed_osc_buffer_size", "\x1b]2;longer\x07", "", .{ max_osc_bytes = 4 } },
        .{ "fixed_size_osc_containing_string_terminator", "\x1b]2;\xe6\x9c\xab\x1b\\", "OSC_DISPATCH:0[]?0{2}{\xe6\x9c\xab}|ESC_DISPATCH:92[]?0|", .{ max_osc_bytes = 5 } },
        .{ "unicode", "\xf0\x9f\x8e\x89_\xf0\x9f\xa6\x80\xf0\x9f\xa6\x80_\xf0\x9f\x8e\x89", "PRINT:127881[]?0|PRINT:95[]?0|PRINT:129408[]?0|PRINT:129408[]?0|PRINT:95[]?0|PRINT:127881[]?0|", .{} },
        .{ "invalid_utf8", "a\xef\xbcb", "PRINT:97[]?0|PRINT:65533[]?0|PRINT:98[]?0|", .{} },
        .{ "partial_utf8", "\xf0\x9f\x9a\x80", "PRINT:128640[]?0|", .{} },
        .{ "partial_utf8_separating_utf8", "\xc4\xb8\xf0\x9f\x8e\x89", "PRINT:312[]?0|PRINT:127881[]?0|", .{} },
        .{ "partial_invalid_utf8", "a\xef\xbcb", "PRINT:97[]?0|PRINT:65533[]?0|PRINT:98[]?0|", .{} },
        .{ "partial_invalid_utf8_split", "\xe4\xbf\x99\xb5", "PRINT:20441[]?0|PRINT:65533[]?0|", .{} },
        .{ "partial_utf8_into_esc", "\xd8\x1b012", "PRINT:65533[]?0|ESC_DISPATCH:48[]?0|PRINT:49[]?0|PRINT:50[]?0|", .{} },
        .{ "c1s", "\x00\x1f\x80\x90\x98\x9b\x9c\x9d\x9e\x9fa", "EXECUTE:0[]?0|EXECUTE:31[]?0|EXECUTE:128[]?0|OSC_DISPATCH:0[]?0{}|", .{ c1 = true } },
        .{ "execute_anywhere", "\x18\x1a", "EXECUTE:24[]?0|EXECUTE:26[]?0|", .{} },
    ];

    expect_equal(cases.count, 34);

    for case: cases {
        for split: 0..case .input.count {
            actual := trace(case .input, split, case .options);
            defer free(actual);
            expect_equal(actual, case .expected, case .name);
        }
    }
} @test
