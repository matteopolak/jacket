//! Mixed scalar/spread normalization regressions.

use jaic::sema::{ProgramSource, VirtualFs};
use test_support::compiler::bare_compiler;

#[path = "support/variadic_source.rs"]
mod source;
use source::SOURCE;
use test_support::bootstrap;

#[test]
fn baked_mixed_type_packs_preserve_relocations_and_deduplicate_instances() {
    let mut c = bootstrap::compiler_with("", VirtualFs::default());
    let source = r"
        pick :: ($types: ..Type) -> Type { return types[2]; }
        forwarded :: ($types: ..Type) -> Type { return pick(u8,..types,s64); }
        identity :: ($types: ..Type) -> bool {
            direct :: #procedure_of_call pick(u8,u16,u32,s64);
            mixed :: #procedure_of_call pick(u8,..types,s64);
            return direct==mixed;
        }
        #assert(forwarded(u16,u32)==u32);
        #assert(identity(u16,u32));
        #assert(!identity(u32,u16));
    ";

    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
}

#[test]
fn baked_mixed_packs_preserve_values_and_order() {
    let mut c = bare_compiler();
    let source = r"
    encode :: ($items: ..int) -> int { result:int; for items result=result*10+it; return result; }
    forward :: ($items: ..int) -> int { return encode(1,..items,4); }
    #assert(forward(2,3)==1234);
    main :: () {}
    ";

    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
}

#[test]
fn scalar_prefix_is_not_discarded_by_a_following_spread() {
    let mut c = bare_compiler();

    if let Err(e)=c.compile_sources(&[ProgramSource::String("encode :: (items: ..int) -> int { result: int; for items result=result*10+it; return result; } test :: () -> int { tail: [2]int = .[2,3]; return encode(1,..tail); } #assert(test()==123);".into())]) {
        panic!("{}",e.render(&c.src.sources));
    }
}

#[test]
fn mixed_segments_preserve_values_order_defaults_and_polymorphism() {
    let mut c = bare_compiler();

    if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", e.render(&c.src.sources));
    }
}

#[test]
fn invalid_spread_lengths_and_c_variadic_arity_are_diagnosed() {
    for (source, expected) in [
        (
            r"
            collect :: (items: ..int) -> int { return items.count; }
            invalid :: () -> int { bad:[]int; bad.count=-1; return collect(1,..bad); }
            #assert(invalid()==0);
        ",
            "variadic",
        ),
        (
            r"
            foreign_call :: (prefix:int, args: ..Any) #foreign;
            main :: () { items:[]int; foreign_call(1,..items); }
        ",
            "spread",
        ),
    ] {
        let mut c = bare_compiler();
        let error = c
            .compile_sources(&[ProgramSource::String(source.into())])
            .expect_err("invalid variadic use must diagnose");

        assert!(
            error.message.contains(expected),
            "{}",
            error.render(&c.src.sources)
        );
    }
}
