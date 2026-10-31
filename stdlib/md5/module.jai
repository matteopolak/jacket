// MD5 digests as hex strings: `md5` for checksums and legacy formats.
//
// ```jai
// digest := md5("abc");
// defer free(digest);
// assert(digest == "900150983cd24fb0d6963f7d28e17f72");
// ```

md5 :: Impl.md5;

#scope_file
Impl :: #import "internal/hash/md5";
