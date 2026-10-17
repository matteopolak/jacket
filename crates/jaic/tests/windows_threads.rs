mod win32_threads {
    //! Host-independent execution of the interpreter's Win32 scheduler API.
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use std::rc::Rc;

    #[test]
    fn win32_and_crt_threads_start_gated_resume_and_keep_their_exit_code() {
        for create in ["CreateThread", "_beginthreadex"] {
            let source = format!(
                r#"
                spawn :: (security:*void, stack_size:u64, entry:*void, argument:*void, flags:u32, id:*u32) -> u64 #foreign libc "{create}";
                libc :: #system_library "libc";
                WaitForSingleObject :: (handle:u64, timeout:u32) -> u32 #foreign;
                ResumeThread :: (handle:u64) -> u32 #foreign;
                SuspendThread :: (handle:u64) -> u32 #foreign;
                CloseHandle :: (handle:u64) -> s32 #foreign;
                GetExitCodeThread :: (handle:u64, code:*u32) -> s32 #foreign;
                GetThreadId :: (handle:u64) -> u32 #foreign;
                callback :: (data:*int) -> u32 #c_call {{ <<data=31; return 17; }}
                check :: () -> int {{
                    value:int;
                    id:u32;
                    handle:=spawn(null,0,cast(*void)callback,*value,4,*id);
                    if handle==0 || id==0 || GetThreadId(handle)!=id return 1;
                    if WaitForSingleObject(handle,0)!=258 || value!=0 return 2;
                    code:u32;
                    if GetExitCodeThread(handle,*code)==0 || code!=259 return 3;
                    if SuspendThread(handle)!=1 || ResumeThread(handle)!=2 return 8;
                    if WaitForSingleObject(handle,1)!=258 || value!=0 return 9;
                    if ResumeThread(handle)!=1 || ResumeThread(handle)!=0 return 4;
                    if WaitForSingleObject(handle,5000)!=0 || value!=31 return 5;
                    if GetExitCodeThread(handle,*code)==0 || code!=17 return 6;
                    if CloseHandle(handle)==0 return 7;
                    return 0;
                }}
                #assert(check()==0);
            "#
            );
            let mut options = Options::host();
            options.runtime_support = false;
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            if let Err(error) = c.compile_sources(&[ProgramSource::String(source)]) {
                panic!("{create}: {}", error.render(&c.sources));
            }
        }
    }

    #[test]
    fn explicit_crt_exit_finishes_only_the_current_interpreted_thread() {
        let source = r#"
            libc :: #system_library "libc";
            spawn :: (security:*void, stack_size:u32, entry:*void, argument:*void, flags:u32, id:*u32) -> u64 #foreign libc "_beginthreadex";
            stop :: (code:u32) #foreign libc "_endthreadex";
            WaitForSingleObject :: (handle:u64, timeout:u32) -> u32 #foreign;
            CloseHandle :: (handle:u64) -> s32 #foreign;
            GetExitCodeThread :: (handle:u64, code:*u32) -> s32 #foreign;
            callback :: (data:*int) -> u32 #c_call { <<data=31; stop(17); <<data=99; return 2; }
            check :: () -> int {
                value:int;
                handle:=spawn(null,0,cast(*void)callback,*value,0,null);
                if handle==0 || WaitForSingleObject(handle,5000)!=0 || value!=31 return 1;
                code:u32;
                if GetExitCodeThread(handle,*code)==0 || code!=17 return 2;
                if CloseHandle(handle)==0 return 3;
                return 0;
            }
            #assert(check()==0);
        "#;
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
            panic!("{}", error.render(&c.sources));
        }
    }

    #[test]
    fn managed_windows_thread_entry_requires_the_real_callback_abi() {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let source = r#"
            CreateThread :: (security:*void, stack_size:u64, entry:*void, data:*void, flags:u32, id:*u32) -> u64 #foreign;
            WaitForSingleObject :: (handle:u64, timeout:u32) -> u32 #foreign;
            CloseHandle :: (handle:u64) -> s32 #foreign;
            wrong :: (data:*void) -> u64 #c_call { return 17; }
            run :: () { h:=CreateThread(null,0,cast(*void)wrong,null,0,null); WaitForSingleObject(h,5000); CloseHandle(h); }
            #run run();
        "#;
        let error = c
            .compile_sources(&[ProgramSource::String(source.into())])
            .expect_err("wide callback result must not masquerade as u32");
        assert!(
            error.message.contains("thread entry ABI"),
            "{}",
            error.render(&c.sources)
        );
    }


    #[test]
    fn unsupported_creation_options_diagnose_instead_of_being_silently_discarded() {
        for (security, stack, flags, expected) in [
            ("cast(*void)*attributes", 0, 0, "security attributes"),
            ("null", 65536, 0, "stack sizes"),
            ("null", 0, 0x10, "creation flags"),
        ] {
            let mut options = Options::host();
            options.runtime_support = false;
            let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
            let source = format!(
                r#"
    CreateThread :: (security:*void,stack:u64,entry:*void,argument:*void,flags:u32,id:*u32)->u64 #foreign;
    worker :: (argument:*void)->u32 #c_call {{ return 1; }}
    Security :: struct {{ length:u32; descriptor:*void; inherit:s32; }}
    #run {{ attributes:Security; attributes.length=cast(u32)size_of(Security); CreateThread({security},{stack},cast(*void)worker,null,{flags},null); }}
    "#
            );
            let error = compiler
                .compile_sources(&[ProgramSource::String(source)])
                .expect_err("unsupported options must not become default OS behavior");
            assert!(
                error.message.contains(expected),
                "{}",
                error.render(&compiler.sources)
            );
        }
    }
}

mod thread_aliases {
    //! Kernel queries through unregistered duplicates must observe real entry completion.
    #![cfg(windows)]
    use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
    use std::rc::Rc;

    #[test]
    fn real_kernel_aliases_preserve_exit_codes_and_handle_lifetimes() {
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(NativeFs));
        let source = format!(
            "{}\n#assert check()==0;",
            include_str!("../../../tests/compiler-semantics/windows-thread-alias-lifecycle.jai")
        );
        if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
            panic!("{}", error.render(&compiler.sources));
        }
    }
}

mod thread_description {
    //! Real Windows thread object semantics for interpreted SDK calls.
    #![cfg(windows)]
    use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
    use std::rc::Rc;

    #[test]
    fn managed_handles_names_queries_pseudo_handles_and_errors_are_native() {
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(NativeFs));
        let source = format!(
            "{}\n#assert check() == 0;",
            include_str!("../../../tests/compiler-semantics/windows-thread-description.jai")
        );
        if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
            panic!("{}", error.render(&compiler.sources));
        }
    }
}
