#import "Basic";

first :: (text: string) -> u8 {
    return text[0];
}

safe_replace :: (repeat: bool) -> u8 {
    name := copy_string("initial");

    while repeat {
        replacement := copy_string(name);
        free(name);
        name = replacement;
    }

    result := first(name);

    free(name);

    return result;
}

safe_transfer :: () -> u8 {
    values: [] u8 = NewArray(8, u8);

    defer free(values.data);
    transferred := values;

    values = .{};
    transferred[0] = 7;
    result := transferred[0];

    free(transferred.data);

    return result;
}

safe_deferred_transfer :: () -> [] u8 {
    values: [] u8 = NewArray(8, u8);
    transferred := false;

    defer if !transferred free(values.data);
    transferred = true;

    return values;
}

bad_old_alias :: () -> u8 {
    name := copy_string("initial");
    old := name;
    replacement := copy_string(name);

    free(name);
    name = replacement;
    result := first(old);

    free(name);

    return result;
}

clear_flag :: (flag: *bool) {
    flag.* = false;
}

bad_exposed_flag :: () -> [] u8 {
    values: [] u8 = NewArray(8, u8);
    transferred := true;

    defer if !transferred free(values.data);
    clear_flag(*transferred);

    return values;
}
