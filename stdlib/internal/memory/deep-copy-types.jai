// Options for `Deep_Copy`: a proc to rewrite every string it copies, and whether to follow
// pointers to structs.
Deep_Copy_Config :: struct {
    string_mapper_proc: (input: string, data: *void) -> (output: string);
    string_mapper_data: *void;
    follow_struct_pointers := true;
}

// Remembers which source pointers were already copied, so shared and cyclic data is copied once.
// Pass one to several `Deep_Copy` calls to keep that sharing across them.
Pointer_Map :: struct {
    table: Table(*void, *void);
}

helper_count := 0;
time_total: float64;
#if DC_DEBUG indent := 0;

#scope_module
#import "Hash_Table";
