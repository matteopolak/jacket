#import "Basic";

consume :: (text: string) -> int {
    if text.count return text.data[0];
    return 0;
}

safe :: (keys: [] u8) {
    for key: keys {
        text: string;
        byte := key;
        if key text = .{ 1, *byte };
        assert(consume(text) >= 0);
    }
}

bad :: (keys: [] u8) -> string {
    text: string;

    for key: keys {
        byte := key;
        text = .{ 1, *byte };
    }

    return text;
}
