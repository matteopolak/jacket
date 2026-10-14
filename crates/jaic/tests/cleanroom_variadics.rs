//! Original mixed scalar/spread normalization regressions.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/variadic_source.rs"]
mod source;
use source::SOURCE;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

#[test]
fn baked_mixed_type_packs_preserve_relocations_and_deduplicate_instances() {
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    let source = r#"
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
    "#;
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn baked_mixed_packs_preserve_values_and_order() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = r#"
    encode :: ($items: ..int) -> int { result:int; for items result=result*10+it; return result; }
    forward :: ($items: ..int) -> int { return encode(1,..items,4); }
    #assert(forward(2,3)==1234);
    main :: () {}
    "#;
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn scalar_prefix_is_not_discarded_by_a_following_spread() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e)=c.compile_sources(&[ProgramSource::String("encode :: (items: ..int) -> int { result: int; for items result=result*10+it; return result; } test :: () -> int { tail: [2]int = .[2,3]; return encode(1,..tail); } #assert(test()==123);".into())]) {
        panic!("{}",e.render(&c.sources));
    }
}

#[test]
fn mixed_segments_preserve_values_order_defaults_and_polymorphism() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn invalid_spread_lengths_and_c_variadic_arity_are_diagnosed() {
    for (source, expected) in [
        (
            r#"
            collect :: (items: ..int) -> int { return items.count; }
            invalid :: () -> int { bad:[]int; bad.count=-1; return collect(1,..bad); }
            #assert(invalid()==0);
        "#,
            "variadic",
        ),
        (
            r#"
            foreign_call :: (prefix:int, args: ..Any) #foreign;
            main :: () { items:[]int; foreign_call(1,..items); }
        "#,
            "spread",
        ),
    ] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let error = c
            .compile_sources(&[ProgramSource::String(source.into())])
            .expect_err("invalid variadic use must diagnose");
        assert!(
            error.message.contains(expected),
            "{}",
            error.render(&c.sources)
        );
    }
}
