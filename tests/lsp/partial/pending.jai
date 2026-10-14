#insert -> string {
    helper :: (value: *Missing_Type) {}
    missing: Missing_Type;
    helper(null);
    return "";
}

#poke_name Missing_Module helper;

healthy :: () -> int {
    local := 42;
    result := local + 1;
    if result <= local return local;
    return result;
}
