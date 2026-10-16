//! Original caller-visible condition value and loop-control regression.
pub const SOURCE: &str = r#"
next :: (state:*u32) -> u32 #no_context { state.*+=1; if state.*==5 return 0; return state.*; }
with_loop :: (body:Code, faults:*u32) #expand {
    state:u32;
    while `condition_value := next(*state) {
        #insert body;
        if condition_value != 7 faults.*+=1;
    }
}
#program_export "named_while_check"
check :: () -> s32 #c_call {
    total:u32;
    faults:u32;
    with_loop(#code {
        if condition_value==2 continue condition_value;
        total=total*10+condition_value;
        if condition_value==3 break condition_value;
        condition_value=7;
    }, *faults);
    if total!=13 return 1;
    if faults!=0 return 3;
    // Ordinary condition names remain visible to their own loop body.
    state:u32;
    while private_value := next(*state) {
        if private_value==2 continue private_value;
        if private_value==3 break private_value;
    }
    if state!=3 return 2;
    return 0;
}
"#;
