is_valid :: (path: string) -> bool {
    for byte: path if byte == 0 return false;
    return true;
}
