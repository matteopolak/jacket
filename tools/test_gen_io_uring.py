import unittest

import gen_io_uring as gen

HEADER = """
#ifndef LINUX_IO_URING_H
#define LINUX_IO_URING_H
#ifdef __cplusplus
extern "C" {
#endif
/* a comment */
#define FLAG_A (1U << 0)
#define FLAG_B	(1U << 3)
#define MASK_ALL	(~0U)
#define BIG		0x80000000ULL
#define COMBINED	(FLAG_A | (__u32)4)
enum op_kind {
	OP_ONE,
	OP_TWO,
	OP_THREE = 7,
	OP_FOUR,
};
enum {
	ANON_X = 1,
};
struct sample {
	__u8 tag;
	__u8 flags;
	__u16 ioprio;
	union {
		__u64 off;
		struct {
			__u32 low;
			__u32 high;
		};
	};
	__u32 len;
	__u8 data[0];
};
struct tail_array {
	__u16 count;
	__DECLARE_FLEX_ARRAY(struct sample, items);
};
#ifdef __cplusplus
}
#endif
#endif
"""


def with_bit() -> str:
    return HEADER


class GeneratorTests(unittest.TestCase):
    def generate(self, header: str = None, width: bool = True) -> str:
        known, widths = dict(gen.KNOWN_SIZES), dict(gen.MACRO_WIDTH)
        gen.KNOWN_SIZES.clear()
        gen.MACRO_WIDTH.clear()
        if width:
            gen.MACRO_WIDTH["MASK_ALL"] = 32
        try:
            return gen.generate(header or with_bit(), "test header")
        finally:
            gen.KNOWN_SIZES.update(known)
            gen.MACRO_WIDTH.clear()
            gen.MACRO_WIDTH.update(widths)

    def test_constants_evaluate_like_c(self):
        out = self.generate()
        self.assertIn("FLAG_A :: 1;", out)
        self.assertIn("FLAG_B :: 8;", out)
        self.assertIn("BIG :: 0x80000000;", out)
        self.assertIn("COMBINED :: 5;", out)
        self.assertIn("ANON_X :: 1;", out)

    def test_negative_or_wide_macros_need_a_declared_width(self):
        with self.assertRaises(gen.GenError):
            self.generate(width=False)
        self.assertIn("MASK_ALL :: 0xffffffff;", self.generate())

    def test_enums_drop_their_common_prefix_and_count_on(self):
        out = self.generate()
        self.assertIn("op_kind :: enum u32 {", out)
        self.assertIn("    ONE :: 0;", out)
        self.assertIn("    THREE :: 7;", out)
        self.assertIn("    FOUR :: 8;", out)

    def test_nested_aggregates_flexible_arrays_and_layout(self):
        out = self.generate()
        self.assertIn("    union {\n        off: u64;\n        struct {\n            low: u32;\n            high: u32;\n        };\n    };", out)
        self.assertIn("    data: [0] u8;", out)
        self.assertIn("    items: [0] sample;", out)
        self.assertIn("#assert size_of(sample) == 24;", out)
        self.assertIn('#assert offset_of(sample, "off") == 8;', out)
        self.assertIn('#assert offset_of(sample, "high") == 12;', out)
        self.assertIn('#assert offset_of(sample, "len") == 16;', out)
        self.assertIn('#assert offset_of(tail_array, "items") == 8;', out)

    def test_a_known_size_that_disagrees_is_an_error(self):
        known, widths = dict(gen.KNOWN_SIZES), dict(gen.MACRO_WIDTH)
        gen.KNOWN_SIZES.clear()
        gen.KNOWN_SIZES["sample"] = 16
        gen.MACRO_WIDTH["MASK_ALL"] = 32
        try:
            with self.assertRaisesRegex(gen.GenError, "kernel ABI is 16"):
                gen.generate(with_bit(), "test header")
        finally:
            gen.KNOWN_SIZES.clear()
            gen.KNOWN_SIZES.update(known)
            gen.MACRO_WIDTH.clear()
            gen.MACRO_WIDTH.update(widths)

    def test_unknown_constructs_stop_the_run(self):
        with self.assertRaises(gen.GenError):
            self.generate(with_bit().replace("__u16 ioprio;", "unsigned long ioprio;"))


if __name__ == "__main__":
    unittest.main()
