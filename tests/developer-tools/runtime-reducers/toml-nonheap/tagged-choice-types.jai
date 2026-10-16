// Original independent types; no TOML application dependency.
Probe_Tag :: enum u8 {
    COUNT :: 3;
    TEXT :: 17;
}

Probe_Choice :: union discriminator: Probe_Tag = .COUNT {
    .COUNT,, quantity: s64 = ---;
    .TEXT,, phrase: string = ---;
}
