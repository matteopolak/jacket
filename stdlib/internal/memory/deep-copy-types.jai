Deep_Copy_Config :: struct {
    string_mapper_proc: (input: string, data: *void) -> (output: string);
    string_mapper_data: *void;
    follow_struct_pointers := true;
}

Pointer_Map :: struct {
    table: Table(*void, *void);
}

helper_count := 0;
time_total: float64;
#if DC_DEBUG indent := 0;

#scope_module
#import "Hash_Table";
