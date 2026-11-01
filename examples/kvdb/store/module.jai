// The storage engine of kvdb: an append-only log of checksummed records plus an in-memory index of
// the live keys. `store_open` replays the log and repairs a torn tail, `store_put` and
// `store_delete` append, `store_compact` rewrites the log with one record per live key.
//
// The command line (../main.jai) and the TCP server (../protocol) both sit on top of this folder,
// and the tests in ../tests exercise it directly. It is imported with `#import,dir "../store"`.

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";
#import "Hash_Table";
#import "md5";

#load "record.jai";
#load "store.jai";
