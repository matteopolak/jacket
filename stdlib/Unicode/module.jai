// Decoding and encoding UTF-8 text, one character at a time.
//
// Strings elsewhere in the standard library are plain bytes; this module is where bytes become code
// points and back. Decoding is lenient unless asked to be `strict`, and the iterator `utf8_iter` is
// always strict. Procedures that allocate use the context allocator and the caller frees the result.
//
// ```jai
// encoded := character_utf32_to_utf8(0xe9);
// defer free(encoded);
// code, size, status := character_utf8_to_utf32(encoded.data, encoded.count, true);
// assert(status == .CONVERSION_OK && code == 0xe9 && size == 2);
// ```

Unicode_Result :: Impl.Unicode_Result;
UNI_REPLACEMENT_CHAR :: Impl.UNI_REPLACEMENT_CHAR;
UNI_MAX_BMP :: Impl.UNI_MAX_BMP;
UNI_MAX_UTF16 :: Impl.UNI_MAX_UTF16;
UNI_MAX_UTF32 :: Impl.UNI_MAX_UTF32;
character_utf8_to_utf32 :: Impl.character_utf8_to_utf32;
character_utf32_to_utf8 :: Impl.character_utf32_to_utf8;
unicode_next_character :: Impl.unicode_next_character;
utf8_next_character :: Impl.utf8_next_character;
utf8_iter :: Impl.utf8_iter;
trailingBytesForUTF8 :: Impl.trailingBytesForUTF8;
utf8_inital_byte_mask :: Impl.utf8_inital_byte_mask;
offsetsFromUTF8 :: Impl.offsetsFromUTF8;
firstByteMark :: Impl.firstByteMark;

#scope_file
Impl :: #import "internal/unicode";
