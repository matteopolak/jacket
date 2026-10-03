Bucket_Array :: Impl.Bucket_Array;
Bucket :: Impl.Bucket;
Bucket_Locator :: Impl.Bucket_Locator;
bucket_array_reset :: Impl.bucket_array_reset;
bucket_array_add :: Impl.bucket_array_add;
bucket_array_find :: Impl.bucket_array_find;
bucket_array_find_pointer :: Impl.bucket_array_find_pointer;
bucket_array_remove :: Impl.bucket_array_remove;
find_and_occupy_empty_slot :: Impl.find_and_occupy_empty_slot;
for_expansion :: Impl.for_expansion;

using,only(.["*[]"]) #import "internal/collections/bucket-array";

#scope_file
Impl :: #import "internal/collections/bucket-array";
