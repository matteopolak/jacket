// Uses `Mode` without declaring it: the importer supplies it with #poke_name.
current: Mode = .b;
set_mode :: (m: Mode) { current = m; }
