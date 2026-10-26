//! `black_box` returns what it is given, for scalars and aggregates, when the interpreter runs it.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

const DECLS: &str = "black_box :: (value: $T) -> T #intrinsic;\n";

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(format!("{DECLS}\n{source}"))]);
    (c, result)
}

#[test]
fn black_box_is_the_identity_for_every_kind_of_value() {
    let (c, result) = compile(
        r#"
        Wide :: struct { a: s64; b: float64; c: [3] u8; text: string; }
        Choice :: enum u8 { LEFT; RIGHT; }
        check :: () -> bool {
            if black_box(5) * 2 != 10 return false;
            if black_box(-7.5) != -7.5 return false;
            if black_box(true) != true return false;
            if black_box(cast(u8) 200) != 200 return false;
            if black_box(Choice.RIGHT) != .RIGHT return false;

            value := 3;
            pointer := black_box(*value);
            if pointer != *value return false;

            wide := Wide.{ a = 9, b = 1.5, c = .[1, 2, 3], text = "boxed" };
            copy := black_box(wide);
            if copy.a != 9 || copy.b != 1.5 || copy.c[2] != 3 || copy.text != "boxed" return false;

            // The result is a copy, not the original.
            copy.a = 10;
            if wide.a != 9 return false;

            empty: Wide;
            return black_box(empty).a == 0;
        }
        #assert(check());
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.src.sources));
    }
}
