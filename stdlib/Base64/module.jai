// Base64 encoding and decoding, in the standard alphabet and the URL-safe one.
//
// Results are allocated with the context allocator and the caller frees them. Decoding reports
// invalid input through a `success` result instead of failing; see `base64_decode`.
//
// ```jai
// encoded := base64_encode("hello");
// defer free(encoded);
// assert(encoded == "aGVsbG8=");
// ```

BASE64_ENCODER_TABLE :: Impl.BASE64_ENCODER_TABLE;
BASE64URL_ENCODER_TABLE :: Impl.BASE64URL_ENCODER_TABLE;
BASE64_DECODER_TABLE :: Impl.BASE64_DECODER_TABLE;
BASE64URL_DECODER_TABLE :: Impl.BASE64URL_DECODER_TABLE;
base64_encode :: Impl.base64_encode;
base64_decode :: Impl.base64_decode;
base64url_encode :: Impl.base64url_encode;
base64url_decode :: Impl.base64url_decode;
test_base64 :: Impl.test_base64;
test_base64_url :: Impl.test_base64_url;
test_base64_module :: Impl.test_base64_module;

#scope_file
Impl :: #import "internal/codecs/base64";
