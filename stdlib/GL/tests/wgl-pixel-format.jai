// Drives the WGL pixel-format selection with stand-in driver calls, so the cases only a real
// Windows GPU driver reaches (a multisampled format with an incomplete descriptor, failing
// sample counts, a window whose format is already set) run on every machine. Each case also
// checks the order of driver calls, which proves which path it took.
#import "Basic";
F :: #import "internal/bindings/wgl-format";

Fake :: struct {
    log: string;
    existing: s32; // what GetPixelFormat reports
    max_samples: s32; // the extended chooser fails above this many samples
    chosen: s32; // the format the extended chooser returns
    legacy: s32; // the format the legacy chooser returns
    complete: bool; // whether DescribePixelFormat reports the capability flags
    usable: bool; // whether the attribute query reports the format usable
    reported: s32; // sample count of the chosen format
}

note :: (fake: *Fake, text: string) {
    fake.log = tprint("%1%2 ", fake.log, text);
}

fake_get_format :: (data: *void, dc: *void) -> s32 {
    fake := cast(*Fake) data;
    note(fake, "get");
    return fake.existing;
}

fake_describe :: (data: *void, dc: *void, format: s32) -> F.Descriptor, bool {
    fake := cast(*Fake) data;
    note(fake, "describe");
    if !fake.complete return .{}, true;
    return .{ flags = 0x1 | 0x4 | 0x20, pixel_type = 0 }, true;
}

fake_choose_legacy :: (data: *void, dc: *void) -> s32 {
    fake := cast(*Fake) data;
    note(fake, "legacy");
    return fake.legacy;
}

fake_set_format :: (data: *void, dc: *void, format: s32) -> bool {
    fake := cast(*Fake) data;
    note(fake, "set");
    return true;
}

fake_choose :: (data: *void, dc: *void, attributes: *s32) -> s32 {
    fake := cast(*Fake) data;
    samples := attributes[17];

    note(fake, tprint("choose:%%", samples, ifx attributes[13] == 16 then "r" else ""));
    if samples > fake.max_samples return 0;

    fake.reported = samples;

    return fake.chosen;
}

fake_query :: (data: *void, dc: *void, format: s32, count: u32, names: *s32, values: *s32) -> bool {
    fake := cast(*Fake) data;

    if count == 4 {
        note(fake, "usable?");

        if fake.usable {
            values[0] = 1;
            values[1] = 1;
            values[2] = 1;
            values[3] = 0x202B;
        }

        return true;
    }

    note(fake, "samples?");
    values[0] = ifx fake.reported > 0 then 1 else 0;
    values[1] = fake.reported;

    return true;
}

driver_for :: (fake: *Fake, extension: bool) -> F.Driver {
    driver: F.Driver = .{
        data = fake,
        get_format = fake_get_format,
        describe = fake_describe,
        choose_legacy = fake_choose_legacy,
        set_format = fake_set_format,
    };

    if extension {
        driver.choose = fake_choose;
        driver.query = fake_query;
    }

    return driver;
}

dc: s64;

expect :: (fake: Fake, selection: F.Selection, ok: bool, format: s32, samples: s32, log: string, what: string) {
    if selection.ok != ok || selection.format != format && ok || selection.samples != samples || fake.log != log {
        print("FAIL %: ok=% format=% samples=% calls=[%]\n", what, selection.ok, selection.format, selection.samples, fake.log);
        print("  expected: ok=% format=% samples=% calls=[%]\n", ok, format, samples, log);
        exit(1);
    }
}

main :: () {
    handle := *dc;

    // (a) The reporter's driver: the extended chooser picks a 4x format whose descriptor lacks
    // the capability flags, but the attribute query reports every capability. Selection must keep
    // the format and never fall back to the legacy chooser.
    {
        fake: Fake = .{ max_samples = 4, chosen = 7, complete = false, usable = true };
        selection := F.select_format(driver_for(*fake, true), handle, true, 4);
        expect(fake, selection, true, 7, 4, "get choose:4 describe set samples? ", "incomplete descriptor from the extended chooser");
    }

    // (b) Sample counts the driver rejects: each failed call only ends that attempt, the count
    // halves, and the last attempt asks for no multisampling.
    {
        fake: Fake = .{ max_samples = 0, chosen = 3, complete = true, usable = true };
        selection := F.select_format(driver_for(*fake, true), handle, true, 4);
        expect(fake, selection, true, 3, 0, "get choose:4 choose:4r choose:2 choose:2r choose:1 choose:1r choose:0 describe set samples? ", "no sample count supported");
    }

    {
        fake: Fake = .{ max_samples = 2, chosen = 4, complete = true, usable = true };
        selection := F.select_format(driver_for(*fake, true), handle, true, 4);
        expect(fake, selection, true, 4, 2, "get choose:4 choose:4r choose:2 describe set samples? ", "steps down to a supported sample count");
    }

    {
        // Without WGL_ARB_multisample the request is not made, whatever was asked for.
        fake: Fake = .{ max_samples = 0, chosen = 6, complete = true, usable = true };
        selection := F.select_format(driver_for(*fake, true), handle, false, 4);
        expect(fake, selection, true, 6, 0, "get choose:0 describe set ", "multisample extension absent");
    }

    // (c) No pixel-format extension: only the legacy chooser runs, and its choice is checked.
    {
        fake: Fake = .{ legacy = 5, complete = true };
        selection := F.select_format(driver_for(*fake, false), handle, false, 4);
        expect(fake, selection, true, 5, 0, "get legacy describe set ", "legacy chooser");
    }

    {
        fake: Fake = .{ legacy = 5, complete = false };
        selection := F.select_format(driver_for(*fake, false), handle, false, 4);
        expect(fake, selection, false, 0, 0, "get legacy describe ", "legacy format with an incomplete descriptor");
    }

    {
        // When the extended chooser finds nothing at all, the legacy chooser takes over.
        fake: Fake = .{ max_samples = -1, legacy = 5, complete = true, usable = true };
        selection := F.select_format(driver_for(*fake, true), handle, true, 0);
        expect(fake, selection, true, 5, 0, "get choose:0 choose:0r legacy describe set samples? ", "extended chooser finds nothing");
    }

    // (d) A window that already has a format: the attribute query decides.
    {
        fake: Fake = .{ existing = 9, complete = true, usable = false };
        selection := F.select_format(driver_for(*fake, true), handle, true, 4);
        expect(fake, selection, false, 0, 0, "get describe usable? ", "existing format the query reports unusable");
    }

    {
        fake: Fake = .{ existing = 9, complete = false, usable = true, reported = 4 };
        selection := F.select_format(driver_for(*fake, true), handle, true, 4);
        expect(fake, selection, true, 9, 4, "get describe usable? samples? ", "existing format with an incomplete descriptor");
    }

    {
        // Without the extension the descriptor is all there is to go on.
        fake: Fake = .{ existing = 9, complete = false };
        selection := F.select_format(driver_for(*fake, false), handle, false, 0);
        expect(fake, selection, false, 0, 0, "get describe ", "existing format, no query, incomplete descriptor");
    }

    print("ok\n");
}
