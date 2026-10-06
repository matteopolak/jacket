// `FRAMES := 3` has no written type: like `FRAMES :: 3` it converts to any integer type
// its value fits, whether the importer passes a value or not.
#module_parameters (FRAMES := 3, LIMIT := 7);

as_count :: (n: u64) -> u64 { return n; }
as_byte :: (n: u8) -> u8 { return n; }

frames_as_count :: () -> u64 { return as_count(FRAMES); }
limit_as_byte :: () -> u8 { return as_byte(LIMIT); }
