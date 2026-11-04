#import "Basic";

saved: string;

make_text :: (count: int) -> string, bool, bool {
    storage: [] u8 = ---;
    storage.data = talloc(count);
    storage.count = count;
    text := to_string(storage.data, storage.count);

    return text, true, false;
}

local_use :: () {
    text, ok, done := make_text(4);
    print("%", text);
}

keep :: () {
    text, ok, done := make_text(4);
    saved = text;
}

reset_then_read :: () {
    text, ok, done := make_text(4);
    reset_temporary_storage();
    print("%", text);
}
