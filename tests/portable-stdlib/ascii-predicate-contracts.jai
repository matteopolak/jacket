#import "Basic";

main :: () {
    for index: 0..255 {
        byte := cast(u8) index;
        letter := (index >= 65 && index <= 90) || (index >= 97 && index <= 122);
        digit := index >= 48 && index <= 57;
        identifier := letter || digit || index == 95;

        if is_alpha(byte) != letter exit(2);
        if is_digit(byte) != digit exit(3);
        if is_alnum(byte) != identifier exit(4);
    }

    if is_alpha(95) || !is_alnum(95) || !is_alpha(65) || !is_alnum(65) || is_alpha(48) || !is_alnum(48) exit(5);

    write_string("ok\n");
}
