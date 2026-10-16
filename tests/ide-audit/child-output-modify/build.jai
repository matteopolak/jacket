#import "Compiler";
#run {
 w:=compiler_create_workspace("child");
 compiler_begin_intercept(w); add_build_file("input.jai",w);
 while true {message:=compiler_wait_for_message(); if message.kind==.COMPLETE break;}
 compiler_end_intercept(w);
 set_build_options_dc(.{do_output=false});
}
