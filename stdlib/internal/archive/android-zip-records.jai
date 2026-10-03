Zip_Compression_Method :: enum u16 {
    NONE :: 0;
    SHRUNK :: 1;
    REDUCED_1 :: 2;
    REDUCED_2 :: 3;
    REDUCED_3 :: 4;
    REDUCED_4 :: 5;
    IMPLODED :: 6;
    RESERVED :: 7;
    DEFLATE :: 8;
    DEFLATE64 :: 9;
    BZIP2 :: 12;
    LZMA :: 14;
    ZSTD :: 93;
    MP3 :: 94;
    XZ :: 95;
    JPEG :: 96;
    WAV_PACK :: 97;
    PPMD :: 98;
}

Zip_Local_File_Header :: struct {
    SIGNATURE: u32 : 0x04034b50;
    signature: u32 = SIGNATURE;
    version_needed_to_extract: u16;
    general_purpose_bit_flag: u16;
    compression_method: Zip_Compression_Method;
    last_mod_file_time: u16;
    last_mod_file_date: u16;
    crc_32: u32 #align 1;
    compressed_size: u32 #align 1;
    uncompressed_size: u32 #align 1;
    filename_length: u16 #align 1;
    extra_field_length: u16 #align 1;
} #no_padding

Zip_Central_Directory_Header :: struct {
    SIGNATURE: u32 : 0x02014b50;
    signature: u32 = SIGNATURE;
    version_made_by: u16;
    version_needed_to_extract: u16;
    general_purpose_bit_flag: u16;
    compression_method: Zip_Compression_Method;
    last_mod_file_time: u16;
    last_mod_file_date: u16;
    crc_32: u32;
    compressed_size: u32;
    uncompressed_size: u32;
    filename_length: u16;
    extra_field_length: u16;
    file_comment_length: u16;
    disk_number_start: u16;
    internal_file_attributes: u16;
    external_file_attributes: u32 #align 1;
    relative_offset_of_local_header: u32 #align 1;
} #no_padding

Zip_End_Of_Central_Directory_Record :: struct {
    SIGNATURE :: 0x06054b50;
    signature: u32 = SIGNATURE;
    disk_number_this: u16;
    disk_number_start: u16;
    total_entries_this_disk: u16;
    total_entries: u16;
    central_directory_size: u32;
    central_directory_offset: u32;
    file_comment_length: u16;
} #no_padding

ZIP_LOCAL_HEADER_SIZE :: size_of(Zip_Local_File_Header);

ZIP_VERSION_DEFAULT: u16 : 0x0014;

#assert size_of(Zip_Local_File_Header) == 30;
#assert size_of(Zip_Central_Directory_Header) == 46;
#assert size_of(Zip_End_Of_Central_Directory_Record) == 22;
