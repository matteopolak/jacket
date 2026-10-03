File_Visit_Info :: struct {
    short_name: string;
    full_name: string;
    modification_time: Apollo_Time;
    size: s64;
    had_error: bool;
    is_symlink: bool;
    is_directory: bool;
    descend_into_directory: bool;
}

#scope_file
#import "Basic";
