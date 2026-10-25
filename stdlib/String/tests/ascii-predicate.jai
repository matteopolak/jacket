#import "Basic";
#import "Extensions/Testing";

ascii_predicates_classify_every_byte :: () {
    for index: 0..255 {
        byte := cast(u8) index;
        letter := (index >= 65 && index <= 90) || (index >= 97 && index <= 122);
        digit := index >= 48 && index <= 57;
        identifier := letter || digit || index == 95;

        expect(is_alpha(byte) == letter, tprint("is_alpha(%)", index));
        expect(is_digit(byte) == digit, tprint("is_digit(%)", index));
        expect(is_alnum(byte) == identifier, tprint("is_alnum(%)", index));
    }
} @test

underscore_is_alphanumeric_but_not_alphabetic :: () {
    expect(!is_alpha(95) && is_alnum(95));
    expect(is_alpha(65) && is_alnum(65));
    expect(!is_alpha(48) && is_alnum(48));
} @test
