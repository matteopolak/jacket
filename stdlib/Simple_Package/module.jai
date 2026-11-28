// Packs named blobs into one file and reads them back: a simple archive for game assets.
//
// - To write, [init] a [Create_Package], [add] each entry and [write] the file.
// - To read, [init_from_file] a [Load_Package] and look entries up by name in its `lookup` table,
//   or read only the names, sizes and offsets with [load_table_of_contents_only].
//
// ```jai,no_run
// #import "Hash_Table";
//
// out: Create_Package;
// init(*out);
// add(*out, "hello.txt", cast([] u8) "hello");
// assert(write(*out, "assets.pack"));
// deinit(*out);
//
// pack: Load_Package;
// assert(init_from_file(*pack, "assets.pack"));
// entry := table_find_pointer(*pack.lookup, "hello.txt");
// assert(cast(string) entry.*.data == "hello");
// ```
//
// The file starts with a 64-byte [Package_Header], then the entry data, then a 64-byte
// [Table_Of_Contents] followed by one record per entry. Readers log an error and return false on
// a truncated or malformed file.

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
free_package :: Impl.free_package;
load_table_of_contents :: Impl.load_table_of_contents;
load_table_of_contents_only :: Impl.load_table_of_contents_only;

#scope_file
Impl :: #import "internal/simple-package";
