Create_Package :: Impl.Create_Package;
Entry_Info :: Impl.Entry_Info;
init :: Impl.init;
add :: Impl.add;
write :: Impl.write;
deinit :: Impl.deinit;
Package_Header :: Impl.Package_Header;
Table_Of_Contents :: Impl.Table_Of_Contents;
Load_Package :: Impl.Load_Package;
KNOWN_HEADER_SIZE :: Impl.KNOWN_HEADER_SIZE;
KNOWN_TABLE_OF_CONTENTS_SIZE :: Impl.KNOWN_TABLE_OF_CONTENTS_SIZE;
init_from_memory :: Impl.init_from_memory;
init_from_file :: Impl.init_from_file;
load_table_of_contents :: Impl.load_table_of_contents;
load_table_of_contents_only :: Impl.load_table_of_contents_only;

#scope_file
Impl :: #import "internal/simple-package";
