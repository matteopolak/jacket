Coff_Raw_Codeview_Info :: struct {
    rsds: [4] u8;
    pdb_guid: Debug_Info.Guid;
    pdb_age: u32;
}

Coff_Codeview_Info :: struct {
    using base: Coff_Raw_Codeview_Info;
    pdb_path: string;
}

#scope_file
Debug_Info :: #import "Debug_Info";
