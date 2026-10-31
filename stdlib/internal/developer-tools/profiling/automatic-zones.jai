// Opens a zone for the rest of the enclosing procedure. The plugin inserts it; a client program
// uses it to time a procedure without writing `zone_end`.
#if IMPORT_MODE == .CLIENT Automatic_Zone :: Automatic_Zone_Guarded;
else Automatic_Zone :: (index: s64) #expand {}

// Opens zone `index` and closes it when the procedure returns, unless a profiler call is already running.
Automatic_Zone_Guarded :: (index: s64) #expand {
    active := !Profile_Context.is_active();
    if active zone_begin(index);
    `defer if active zone_end();
}
