#import "Basic";

release_data :: (bytes: [] u8) {
    context.allocator.proc(.FREE, 0, 0, bytes.data, context.allocator.data);
}

main :: () {
    text := copy_string("owned");
    old := text.data;
    release_data(cast([] u8) text);
    print("%", old[0]);
}
