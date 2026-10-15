// CRC and Base64 public vectors and permissive decode edge cases.
#import "Basic";
Base64 :: #import "Base64";
Crc :: #import "Crc";

main :: () {
    assert(Crc.crc64("") == 0);
    assert(Crc.crc64("123456789") == 0x62ec59e3f1a4f00a);
    assert(Base64.BASE64_DECODER_TABLE[10] == 64);
    assert(Base64.BASE64_DECODER_TABLE[61] == 65);
    assert(Base64.BASE64_DECODER_TABLE[0] == 66);

    assert(Base64.base64_encode("Man") == "TWFu");
    assert(Base64.base64_encode("M") == "TQ==");
    assert(Base64.base64_encode("M", pad = false) == "TQ");
    assert(Base64.base64url_encode("??") == "Pz8");
    url_text, valid := Base64.base64url_decode("Pz8");

    assert(valid && url_text == "??");

    text, valid := Base64.base64_decode("TWE");

    assert(valid && text == "Ma");
    text, valid = Base64.base64_decode("T");
    assert(valid && text == "");
    text, valid = Base64.base64_decode("TWE=ignored");
    assert(valid && text == "Ma");
    text, valid = Base64.base64_decode("T!WE");
    assert(!valid && text == "");
    text, valid = Base64.base64_decode("TQ\n==");
    assert(valid && text == "M");
    print("ok\n");
}
