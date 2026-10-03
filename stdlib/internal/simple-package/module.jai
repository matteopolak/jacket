#load "records.jai";
#load "create.jai";
#load "read.jai";

#scope_module
B :: #import "Basic";
Files :: #import "File";
Pools :: #import "Pool";
HT :: #import "Hash_Table";
Strings :: #import "String";
Wire :: #import "internal/resource-wire/integers";

HEADER_MAGIC: u32 : 0x706d6973;
TOC_MAGIC: u32 : 0x21636f74;
