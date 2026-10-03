//! Literal C strings have real static backing, independent of caller frames.
#[path = "support/checked_execution.rs"]
#[allow(
    dead_code,
    reason = "The shared harness exposes additional capability runners."
)]
mod checked_execution;

#[test]
fn literal_bytes_and_the_appended_nul_agree_with_native_execution() {
    checked_execution::check(
        r#"
        inspect::(bytes:*u8)->int {
            if bytes[0]!=65 || bytes[1]!=0 || bytes[2]!=66 || bytes[3]!=0 return 1;
            return 42;
        }
        main::()->int { return inspect("A\0B"); }
        "#,
        42,
    );
    checked_execution::check(
        r#"
        main::()->int {
            empty:*u8=""; terminated:*u8="A\0";
            if empty[0]!=0 || terminated[0]!=65 || terminated[1]!=0 || terminated[2]!=0 return 1;
            return 42;
        }
        "#,
        42,
    );
}

#[test]
fn returning_a_literal_pointer_does_not_borrow_the_callee_frame() {
    checked_execution::check(
        r#"
        name::()->*u8 { return "A"; }
        main::()->int {
            bytes:=name();
            if bytes[0]==65 && bytes[1]==0 return 42;
            return 1;
        }
        "#,
        42,
    );
}

#[test]
fn generic_pointer_arguments_infer_the_literals_actual_byte_element() {
    checked_execution::check(include_str!("fixtures/c-string-generic.jai.pending"), 42);
}

#[test]
fn literal_arguments_reach_trusted_c_and_stdio_at_both_optimization_levels() {
    checked_execution::check_foreign_execution(
        r#"
        verify::(bytes:*u8)->s32 #foreign "verify";
        FILE::struct {}
        fopen::(path:*u8,mode:*u8)->*FILE #foreign "fopen";
        fclose::(file:*FILE)->s32 #foreign "fclose";
        fputc::(value:s32,file:*FILE)->s32 #foreign "fputc";
        fgetc::(file:*FILE)->s32 #foreign "fgetc";
        rewind::(file:*FILE) #foreign "rewind";
        main::()->int {
            if verify("A\0B")!=42 return 1;
            file:=fopen("literal-stdio.bin","w+b");
            if !file return 2;
            if fputc(41,file)!=41 return 3;
            rewind(file);
            result:=fgetc(file);
            if fclose(file)!=0 return 4;
            return result+1;
        }
        "#,
        "#include <stdint.h>\nint32_t verify(const unsigned char *p) { return p[0]==65 && p[1]==0 && p[2]==66 && p[3]==0 ? 42 : 1; }\n",
        42,
    );
}
