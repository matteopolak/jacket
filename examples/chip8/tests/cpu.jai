#import "Basic";
#import "Extensions/Testing";
#import,dir "../core";

fresh :: () -> Machine {
    m: Machine;
    reset(*m, "\x00\xFD");
    return m;
}

control_flow :: (t: *Test) {
    m := fresh();

    execute(*m, 0x1234);
    expect_equal(m.pc, 0x234);
    execute(*m, 0x2456);
    expect_equal(m.pc, 0x456);
    expect_equal(m.sp, 1);
    execute(*m, 0x00EE);
    expect_equal(m.pc, 0x234);
    expect_equal(m.sp, 0);
    execute(*m, 0x00FD);
    expect(m.halted);
} @test

skip_opcodes :: (t: *Test) {
    m := fresh();
    m.v[1] = 9;
    m.v[2] = 9;
    execute(*m, 0x3109);
    expect_equal(m.pc, 0x202);
    execute(*m, 0x4108);
    expect_equal(m.pc, 0x204);
    execute(*m, 0x5120);
    expect_equal(m.pc, 0x206);
    execute(*m, 0x9120);
    expect_equal(m.pc, 0x206);
    m.v[2] = 8;
    execute(*m, 0x9120);
    expect_equal(m.pc, 0x208);
    execute(*m, 0x3108);
    execute(*m, 0x4109);
    execute(*m, 0x5120);
    expect_equal(m.pc, 0x208);
} @test

load_add_and_index :: (t: *Test) {
    m := fresh();

    execute(*m, 0x61FE);
    execute(*m, 0x7104);
    expect_equal(m.v[1], 2);
    execute(*m, 0xA321);
    expect_equal(m.i, 0x321);
    execute(*m, 0xF11E);
    expect_equal(m.i, 0x323);
    m.i = 4095;
    execute(*m, 0xF11E);
    expect_equal(m.i, 1);
} @test

arithmetic_opcodes :: (t: *Test) {
    ops :: int.[0, 1, 2, 3, 4, 5, 6, 7, 14];
    results :: int.[7, 15, 0, 15, 15, 1, 4, 255, 16];

    for ops {
        m := fresh();
        m.v[1] = 8;
        m.v[2] = 7;
        execute(*m, 0x8120 | it);
        expect_equal(cast(int) m.v[1], results[it_index]);
        expect(!m.halted);
    }
} @test

carry_borrow_and_vf_alias :: (t: *Test) {
    m := fresh();
    m.v[1] = 255;
    m.v[2] = 1;
    execute(*m, 0x8124);
    expect_equal(m.v[1], 0);
    expect_equal(m.v[15], 1);
    execute(*m, 0x8125);
    expect_equal(m.v[1], 255);
    expect_equal(m.v[15], 0);
    m.v[1] = 1;
    execute(*m, 0x8125);
    expect_equal(m.v[15], 1);
    m.v[15] = 255;
    execute(*m, 0x8F24);
    expect_equal(m.v[15], 1);
    m.v[1] = 128;
    execute(*m, 0x810E);
    expect_equal(m.v[1], 0);
    expect_equal(m.v[15], 1);
} @test

quirk_presets :: (t: *Test) {
    expect(CHIP8.shift_vy && CHIP8.increment_i && CHIP8.display_wait && CHIP8.clear_vf);
    expect(!CHIP8.jump_vx && CHIP8.clip);
    expect(SUPER_CHIP.jump_vx && !SUPER_CHIP.shift_vy && !SUPER_CHIP.increment_i);
    expect(!MODERN.jump_vx && !MODERN.shift_vy && !MODERN.display_wait);

    for preset: Quirks.[CHIP8, SUPER_CHIP, MODERN] {
        m := fresh();
        m.quirks = preset;
        m.v[1] = 2;
        m.v[2] = 8;
        execute(*m, 0x8126);
        expect_equal(m.v[1], ifx preset.shift_vy then 4 else 1);
        m.v[0] = 3;
        m.v[2] = 7;
        execute(*m, 0xB200);
        expect_equal(m.pc, ifx preset.jump_vx then 0x207 else 0x203);
        m.i = 0x300;
        execute(*m, 0xF255);
        expect_equal(m.i, ifx preset.increment_i then 0x303 else 0x300);
        m.v[15] = 9;
        execute(*m, 0x8121);
        expect_equal(m.v[15], ifx preset.clear_vf then 0 else 9);
    }
} @test

bcd_and_register_memory :: (t: *Test) {
    m := fresh();
    m.i = 0x300;
    m.v[3] = 237;
    execute(*m, 0xF333);
    expect_equal(m.memory[0x300], 2);
    expect_equal(m.memory[0x301], 3);
    expect_equal(m.memory[0x302], 7);
    for 0..15 m.v[it] = cast(u8) (it * 3);

    execute(*m, 0xFF55);
    for *m.v it.* = 0;

    execute(*m, 0xFF65);
    for 0..15 expect_equal(cast(int) m.v[it], it * 3);

    m.quirks = CHIP8;
    m.i = 0x300;
    execute(*m, 0xF265);
    expect_equal(m.i, 0x303);
} @test

fonts_and_rpl :: (t: *Test) {
    m := fresh();
    m.v[0] = 10;
    execute(*m, 0xF029);
    expect_equal(m.i, 0x82);
    expect_equal(m.memory[m.i], 0xF0);
    execute(*m, 0xF030);
    expect_equal(m.i, 0x104);
    expect_equal(m.memory[m.i], 0xFF);
    execute(*m, 0xF075);
    m.v[0] = 0;
    execute(*m, 0xF085);
    expect_equal(m.v[0], 10);
    execute(*m, 0xF875);
    expect(m.halted);
} @test

timer_opcodes_and_sixty_hz :: (t: *Test) {
    m := fresh();
    m.v[1] = 120;
    execute(*m, 0xF115);
    execute(*m, 0xF118);
    execute(*m, 0xF207);
    expect_equal(m.v[2], 120);
    advance_time(*m, 16666666);
    expect_equal(m.delay, 120);
    advance_time(*m, 1);
    expect_equal(m.delay, 119);
    advance_time(*m, 983333333);
    expect_equal(m.delay, 60);
    expect_equal(m.sound, 60);
    advance_time(*m, 10000000000);
    expect_equal(m.delay, 0);
    expect_equal(m.sound, 0);
} @test

timers_do_not_follow_cpu_rate :: (t: *Test) {
    for rate: int.[1, 12, 120] {
        m := fresh();
        m.memory[0x200] = 0x12;
        m.memory[0x201] = 0;
        m.delay = 120;

        for 1..60 {
            step(*m, rate);
            advance_time(*m, 16666667);
        }

        expect_equal(m.delay, 60);
    }
} @test

key_skip_and_wait_for_release :: (t: *Test) {
    m := fresh();
    m.v[1] = 5;
    set_key(*m, 5, true);
    execute(*m, 0xE19E);
    expect_equal(m.pc, 0x202);
    execute(*m, 0xE1A1);
    expect_equal(m.pc, 0x202);
    set_key(*m, 5, false);
    execute(*m, 0xE1A1);
    expect_equal(m.pc, 0x204);
    execute(*m, 0xF20A);
    expect_equal(step(*m, 10), 0);
    set_key(*m, 9, true);
    expect_equal(m.waiting_key, 2);
    set_key(*m, 9, false);
    expect_equal(m.waiting_key, -1);
    expect_equal(m.v[2], 9);
} @test

random_is_seeded_and_masked :: (t: *Test) {
    a := fresh();
    b := fresh();
    different := false;

    for 1..20 {
        execute(*a, 0xC1FF);
        execute(*b, 0xC1FF);
        expect_equal(a.v[1], b.v[1]);
        if a.v[1] != 0 different = true;

        execute(*a, 0xC20F);
        execute(*b, 0xC20F);
        expect(a.v[2] < 16);
    }

    expect(different);
} @test

drawing_collision_clip_wrap_and_wait :: (t: *Test) {
    m := fresh();
    m.i = 0x300;
    m.memory[m.i] = 0xFF;
    m.v[1] = 63;
    m.v[2] = 31;
    execute(*m, 0xD121);
    expect_equal(m.pixels[31 * 128 + 63], 1);
    expect_equal(m.pixels[31 * 128], 0);
    execute(*m, 0xD121);
    expect_equal(m.v[15], 1);
    expect_equal(m.pixels[31 * 128 + 63], 0);
    m.quirks.clip = false;
    execute(*m, 0xD121);
    expect_equal(m.pixels[31 * 128], 1);
    m.quirks = CHIP8;
    execute(*m, 0xD121);
    expect(m.draw_blocked);
    expect_equal(step(*m, 3), 0);
    advance_time(*m, 16666667);
    expect(!m.draw_blocked);
    execute(*m, 0x00E0);
    for m.pixels expect_equal(it, 0);
} @test

super_chip_modes_scroll_and_wide_sprite :: (t: *Test) {
    m := fresh();

    execute(*m, 0x00FF);
    expect(m.high);
    expect_equal(width(*m), 128);
    m.i = 0x300;
    m.memory[m.i] = 0x80;
    m.memory[m.i + 1] = 1;
    execute(*m, 0xD010);
    expect_equal(m.pixels[0], 1);
    expect_equal(m.pixels[15], 1);
    execute(*m, 0x00FB);
    expect_equal(m.pixels[4], 1);
    expect_equal(m.pixels[0], 0);
    execute(*m, 0x00FC);
    expect_equal(m.pixels[0], 1);
    execute(*m, 0x00C3);
    expect_equal(m.pixels[128 * 3], 1);
    expect_equal(m.pixels[0], 0);
    execute(*m, 0x00FE);
    expect(!m.high);
    for m.pixels expect_equal(it, 0);
} @test

bad_programs_halt_without_out_of_bounds :: (t: *Test) {
    for op: int.[0x0000, 0x5121, 0x9121, 0x8128, 0xE100, 0xF100, 0x00EE] {
        m := fresh();
        execute(*m, op);
        expect(m.halted);
        expect(m.error.count > 0);
    }

    for op: int.[0xF133, 0xFF55, 0xFF65, 0xD010] {
        m := fresh();
        m.i = 4095;
        execute(*m, op);
        expect(m.halted);
    }

    m := fresh();
    m.pc = 4095;
    expect_equal(step(*m, 1), 0);
    expect(m.halted);
    m = fresh();
    m.sp = 16;
    execute(*m, 0x2200);
    expect(m.halted);
    expect(!reset(*m, ""));
} @test

step_fetches_big_endian_and_honours_budget :: (t: *Test) {
    m: Machine;

    require(reset(*m, "\x61\x07\x71\x01\x00\xFD"));
    expect_equal(step(*m, 2), 2);
    expect_equal(m.v[1], 8);
    expect_equal(m.pc, 0x204);
    expect_equal(step(*m, 10), 1);
    expect(m.halted);
    expect_equal(m.error, "");
} @test

framebuffer_meow_changes_and_repeats :: (t: *Test) {
    {
        m := fresh();
        low, high := frame_hash(*m);
        again, again_hi := frame_hash(*m);

        expect_equal(low, again);
        expect_equal(high, again_hi);
        m.pixels[0] = 1;
        changed, changed_hi := frame_hash(*m);

        expect(low != changed || high != changed_hi);
    }
} @test
