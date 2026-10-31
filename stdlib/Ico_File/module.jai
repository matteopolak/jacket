// Makes Windows ICO files from pictures: `create_ico_file` from pixels, or
// `create_ico_file_from_bitmap_filename` from an image file. Each icon holds five sizes, 256 down to 16.
create_ico_file :: Encode.create_ico_file;
create_ico_file_from_bitmap_filename :: Load.create_ico_file_from_bitmap_filename;
get_bytes_for_image :: Encode.get_bytes_for_image;
get_total_bytes_for_size :: Encode.get_total_bytes_for_size;

#scope_file
Encode :: #import "internal/resource-wire/ico-encode";
Load :: #import "internal/resource-wire/ico-load";
