//! Native thread entry signatures are checked before allocating an OS worker.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[test]
fn posix_start_routines_require_one_pointer_argument_and_result_with_c_abi() {
    for worker in [
        "worker :: (argument:*void) #c_call {}",
        "worker :: (argument:*void)->u64 #c_call {return 17;}",
        "worker :: (argument:u32)->*void #c_call {return null;}",
        "worker :: (argument:*void)->*void {return null;}",
    ] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        let source = format!(
            r#"
pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
{worker}
#run {{thread:u64;pthread_create(*thread,null,cast(*void)worker,null);}}
"#
        );
        let error = compiler
            .compile_sources(&[ProgramSource::String(source)])
            .expect_err("an invalid start routine must not become a zero-result worker");
        assert!(
            error.message.contains("thread entry ABI"),
            "{}",
            error.render(&compiler.sources)
        );
    }
}


#[test]
fn typed_pointer_callbacks_return_the_actual_pointer_and_mutation_to_joiners() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = r#"
pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
pthread_join :: (thread:u64,result:**void)->s32 #foreign;
worker :: (value:*int)->*void #c_call { value.* += 1; return value; }
check :: ()->bool #no_context {
    value:int=73; thread:u64; result:*void;
    if pthread_create(*thread,null,cast(*void)worker,*value)!=0 return false;
    if pthread_join(thread,*result)!=0 return false;
    return value==74 && result==cast(*void)*value;
}
#assert check();
"#;
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
}
