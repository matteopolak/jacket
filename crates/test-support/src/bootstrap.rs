//! A deliberately scoped prelude for tests: the reflection fields they need, not a substitute for
//! integrated prelude verification. Needs the `compiler` feature.

/// The prelude source.
pub const BOOTSTRAP: &str = r"
Type_Info :: struct { type: u32; runtime_size: s64; }
Type_Info_Integer :: struct { using #as info: Type_Info; signed: bool; }
Type_Info_String :: struct { using #as info: Type_Info; }
Type_Info_Float :: struct { using #as info: Type_Info; }
Type_Info_Void :: struct { using #as info: Type_Info; }
Type_Info_Pointer :: struct { using #as info: Type_Info; pointer_to: *Type_Info; }
Type_Info_Struct_Member :: struct {
    name: string; type: *Type_Info; offset_in_bytes: s64;
    flags: u32; notes: [] string; offset_into_constant_storage: s64 = -1;
}
Type_Info_Struct :: struct {
    using #as info: Type_Info; name: string;
    parameters: [] Type_Info_Struct_Member; specified_parameters: [] Type_Info_Struct_Member;
    members: [] Type_Info_Struct_Member;
    status_flags: u16; nontextual_flags: u16; textual_flags: u16; alignment: s32;
    polymorph_source_struct: *Type_Info_Struct;
    initializer: (*void) #no_context; constant_storage: [] u8; notes: [] string;
}
";

/// A compiler that preloads [`BOOTSTRAP`] followed by `extra`, over `fs` (which may hold more
/// modules), with the runtime support module off. The prelude is `/lib/prelude/module.jai`.
pub fn compiler_with(extra: &str, mut fs: jaic::sema::VirtualFs) -> jaic::sema::Compiler {
    let mut options = jaic::sema::Options::host();

    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    fs.insert(
        "/lib/prelude/module.jai",
        format!("{BOOTSTRAP} {extra}").as_bytes(),
    );
    jaic::sema::Compiler::new(options, std::rc::Rc::new(fs))
}

/// Compile `source` as one string program over [`BOOTSTRAP`]; the compiler is returned for
/// inspection.
pub fn compile(source: &str) -> (jaic::sema::Compiler, jaic::sema::Result<()>) {
    let mut c = compiler_with("", jaic::sema::VirtualFs::default());
    let result = c.compile_sources(&[jaic::sema::ProgramSource::String(source.into())]);

    (c, result)
}

/// Compile `source` as `/app/main.jai` over [`BOOTSTRAP`].
pub fn compile_app(source: &str) -> (jaic::sema::Compiler, jaic::sema::Result<()>) {
    let mut fs = jaic::sema::VirtualFs::default();

    fs.insert("/app/main.jai", source.as_bytes());
    let mut c = compiler_with("", fs);
    let result = c.compile_program(std::path::Path::new("/app/main.jai"));

    (c, result)
}
