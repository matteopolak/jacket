// Picks the pixel format for a window's device context. Every driver call goes through `Driver`, so
// the choice can run against a stand-in on machines that have no OpenGL driver, or no Windows.
Descriptor :: struct {
    flags: u32;
    pixel_type: u8;
}

// Each procedure receives `data` first. A nil `choose` or `query` means the driver lacks the
// pixel-format extension.
Driver :: struct {
    data: *void;
    get_format: (data: *void, dc: *void) -> s32;
    describe: (data: *void, dc: *void, format: s32) -> Descriptor, bool;
    choose_legacy: (data: *void, dc: *void) -> s32;
    set_format: (data: *void, dc: *void, format: s32) -> bool;
    choose: (data: *void, dc: *void, attributes: *s32) -> s32;
    query: (data: *void, dc: *void, format: s32, count: u32, names: *s32, values: *s32) -> bool;
}

Selection :: struct {
    ok: bool;
    format: s32;
    samples: s32;
}

// Settles on the window's pixel format, installing one when it has none, and reports the sample
// count the format really has.
select_format :: (driver: Driver, dc: *void, multisample: bool, wanted_samples: s32) -> Selection {
    failed: Selection;
    format := driver.get_format(driver.data, dc);

    if format == 0 {
        samples := ifx multisample then wanted_samples else 0;

        // Walk down from the requested sample count, then relax the colour
        // requirements, before settling on the legacy chooser. Some drivers
        // report an unsupported sample count as a failed call, not an empty
        // result, so a failure only ends the current attempt.
        while driver.choose {
            format = choose_extended(driver, dc, multisample, samples, false);
            if format == 0 format = choose_extended(driver, dc, multisample, samples, true);
            if format != 0 || samples == 0 break;

            samples /= 2;
        }

        // The extended chooser only returns formats that have every requested attribute, and some
        // drivers describe those formats incompletely, so only the legacy chooser's nearest match
        // is checked.
        extended := format != 0;

        if !extended {
            format = driver.choose_legacy(driver.data, dc);
            if format == 0 return failed;
        }

        descriptor, described := driver.describe(driver.data, dc, format);
        if !described return failed;
        if !extended && !valid_descriptor(descriptor) return failed;
        if !driver.set_format(driver.data, dc, format) return failed;
    } else {
        descriptor, described := driver.describe(driver.data, dc, format);
        if !described return failed;
        if !usable_format(driver, dc, format, descriptor) return failed;
    }

    result: Selection = .{ ok = true, format = format };

    if multisample && driver.query {
        names: [2] s32 = .[0x2041, 0x2042];
        values: [2] s32;
        if !driver.query(driver.data, dc, format, 2, names.data, values.data) return failed;
        if values[0] != 0 result.samples = values[1];
    }

    return result;
}

#scope_file

// Asks the pixel-format extension for a double buffered RGBA window format.
// The strict form wants 8-bit alpha and 24-bit depth; the relaxed form takes any
// colour depth with at least 16 depth bits. Returns 0 when nothing matches.
choose_extended :: (driver: Driver, dc: *void, multisample: bool, samples: s32, relaxed: bool) -> s32 {
    attributes: [19] s32 = .[0x2001, 1, 0x2010, 1, 0x2011, 1, 0x2013, 0x202B, 0x2014, 24, 0x201B, 8, 0x2022, 24, 0, 0, 0, 0, 0];

    if relaxed {
        attributes[8] = 0;
        attributes[9] = 0;
        attributes[10] = 0;
        attributes[11] = 0;
        attributes[13] = 16;
    }

    if multisample {
        attributes[14] = 0x2041;
        attributes[15] = ifx samples > 0 then 1 else 0;
        attributes[16] = 0x2042;
        attributes[17] = samples;
    }

    return driver.choose(driver.data, dc, attributes.data);
}

// A format already on the window may have come from the extended chooser, so ask the driver for its
// attributes when it can answer instead of trusting the descriptor.
usable_format :: (driver: Driver, dc: *void, format: s32, descriptor: Descriptor) -> bool {
    if !driver.query return valid_descriptor(descriptor);

    names: [4] s32 = .[0x2001, 0x2010, 0x2011, 0x2013];
    values: [4] s32;

    if !driver.query(driver.data, dc, format, 4, names.data, values.data) return valid_descriptor(descriptor);

    return values[0] != 0 && values[1] != 0 && values[2] != 0 && values[3] == 0x202B;
}

valid_descriptor :: (descriptor: Descriptor) -> bool {
    needed: u32 = 0x1 | 0x4 | 0x20; // double buffer, draw to window, support OpenGL
    return (descriptor.flags & needed) == needed && descriptor.pixel_type == 0; // RGBA
}
