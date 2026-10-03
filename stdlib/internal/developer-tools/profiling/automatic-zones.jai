#if IMPORT_MODE == .CLIENT Automatic_Zone :: Automatic_Zone_Guarded;
else Automatic_Zone :: (index: s64) #expand {}

Automatic_Zone_Guarded :: (index: s64) #expand {
    active := !Profile_Context.is_active();
    if active zone_begin(index);
    `defer if active zone_end();
}
