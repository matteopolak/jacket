// Filled in by the metaprogram, into this module's scope.
#placeholder Kind_Names;
#placeholder Kind_Labels;
#placeholder Kind;

// `Kind` is generated from `Kind_Names`, so it only exists after the metaprogram ran.
#insert -> string {
    text := "Kind :: enum u8 { NONE :: 0; ";
    for 1..Kind_Names.count - 1 {
        text = sprint("%1%2 :: %3; ", text, Kind_Names[it], it);
    }
    return sprint("%}", text);
};
