// The fixed part of a CodeView PDB 7.0 record: the `RSDS` signature, the PDB GUID and the PDB age.
Coff_Raw_Codeview_Info :: struct {
    rsds: [4] u8;
    pdb_guid: Debug_Info.Guid;
    pdb_age: u32;
}

// A CodeView PDB 7.0 record: the fixed part in `base` followed by the PDB path.
Coff_Codeview_Info :: struct {
    using base: Coff_Raw_Codeview_Info;
    pdb_path: string;
}

#scope_file
Debug_Info :: #import "Debug_Info";
