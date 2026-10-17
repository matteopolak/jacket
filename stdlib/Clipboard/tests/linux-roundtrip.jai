#import "Basic";
#import "Clipboard";

main :: () {
    original := "Clipboard: café 🌍\0tail";
    storage := copy_string(original);

    os_clipboard_set_text(storage);
    // The selection must own a snapshot, independent of the caller allocator.
    memset(storage.data, 33, storage.count);
    free(storage);
    actual := os_clipboard_get_text();

    assert(actual == original);
    free(actual);
    os_clipboard_set_text("");
    actual = os_clipboard_get_text();
    assert(actual.count == 0);
    free(actual);
    print("ok\n");
}
