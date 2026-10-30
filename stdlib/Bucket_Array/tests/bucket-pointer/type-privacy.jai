T :: #import "Type_Only";
H :: #import "Private";
C :: #import "Bucket_Array";

#assert type_info(T.Bucket_Array(s64, 2)) == type_info(C.Bucket_Array(s64, 2));
#assert type_info(H.Bucket_Array(s64, 2)) == type_info(C.Bucket_Array(s64, 2));
#assert !#exists(H.Actual);
#assert !#exists(C.Impl);
main :: () {}
