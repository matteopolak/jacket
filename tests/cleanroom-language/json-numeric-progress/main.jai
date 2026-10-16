#import "Basic";
#import "Extensions/Json";

Case :: struct {
    text: string;
    bits: u64;
}

main :: () {
    cases := Case.[
        .{ "0.1", 0x3fb999999999999a },
        .{ "9007199254740993", 0x4340000000000000 },
        .{ "1.7976931348623157e308", 0x7fefffffffffffff },
        .{ "2.2250738585072014e-308", 0x0010000000000000 },
        .{ "4.9e-324", 1 },
        .{ "2.4703282292062327e-324", 0 },
        .{ "2.4703282292062328e-324", 1 },
        .{ "-0", 0x8000000000000000 }
    ];

    print("ready\n");

    for case: cases {
        for 1..10 {
            value, error := from_json(float64, case .text);
            assert(error.kind == .NONE);
            assert(cast,force(u64) value == case .bits);
        }

        print("parsed %\n", case .text);
    }

    for case: cases {
        value := cast,force(float64) case .bits;

        for 1..10 {
            encoded := to_json(value);
            decoded, error := from_json(float64, encoded);
            assert(error.kind == .NONE);
            assert(cast,force(u64) decoded == case .bits);
        }

        print("roundtrip %\n", case .text);
    }

    print("done\n");
}
