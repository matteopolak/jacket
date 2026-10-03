Single_Rewrite :: struct {
    l0, c0, l1, c1: s32;
    new_text: string;
}

File_To_Rewrite :: struct {
    filename: string;
    single_rewrites: [..] Single_Rewrite;
    full_data: string;
    lines: [] string;
}

My_Plugin :: struct {
    #as using base: Metaprogram_Plugin;
    files_to_rewrite: Table(*Message_File, File_To_Rewrite);
    write_mode := false;
}

#load "../internal/developer-tools/rewrite.jai";

#scope_file
#import "Compiler";
#import "Hash_Table";
