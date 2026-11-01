// Deterministic CHIP-8/SUPER-CHIP machine. Timers advance only through advance_time.
#import "Basic";
#import "PCG";
M :: #import "Extensions/Meow_Portable";

Quirks :: struct {
    shift_vy: bool;
    jump_vx: bool;
    increment_i: bool;
    display_wait: bool;
    clip: bool = true;
    clear_vf: bool;
}

CHIP8 :: Quirks.{ shift_vy = true, increment_i = true, display_wait = true, clear_vf = true };
SUPER_CHIP :: Quirks.{ jump_vx = true };
MODERN :: Quirks.{};

Machine :: struct {
    memory: [4096] u8;
    v: [16] u8;
    i: int;
    pc: int = 0x200;
    stack: [16] int;
    sp: int;
    delay: u8;
    sound: u8;
    pixels: [8192] u8; // 128-byte stride in both modes, inactive cells zero
    keys: [16] bool;
    rpl: [8] u8;
    high: bool;
    halted: bool;
    error: string;
    quirks: Quirks;
    rng: u64;
    timer_units: u64; // nanoseconds * 60, remainder modulo 1e9
    draw_blocked: bool;
    waiting_key: int = -1;
    released_key: int = -1;
}

FONT :: u8.[
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80
];

// Resets all state and copies a ROM. Large glyphs are our small glyphs doubled horizontally/vertically.
reset :: (m: *Machine, rom: string, quirks := MODERN, seed: u32 = 1) -> bool {
    m.* = .{};
    m.quirks = quirks;
    random_seed(*m.rng, seed);
    for FONT m.memory[0x50 + it_index] = it;

    for digit: 0..15 for row : 0..4 {
        bits := FONT[digit * 5 + row];
        wide: u8;

        for bit: 0..3 if (bits & (0x80 >> bit)) != 0 wide |= cast(u8) (0xC0 >> (bit * 2));

        m.memory[0xA0 + digit * 10 + row * 2] = wide;
        m.memory[0xA0 + digit * 10 + row * 2 + 1] = wide;
    }

    if rom.count == 0 || rom.count > 3584 return fail(m, "ROM size must be 1..3584 bytes");
    for rom m.memory[0x200 + it_index] = it;

    return true;
}

fail :: (m: *Machine, message: string) -> bool {
    m.error = message;
    m.halted = true;
    return false;
}

width :: (m: *Machine) -> int {
    return ifx m.high then 128 else 64;
}

height :: (m: *Machine) -> int {
    return ifx m.high then 64 else 32;
}

// Separate from instruction rate: an exact integer timebase at 60 Hz, also releases display wait.
advance_time :: (m: *Machine, nanoseconds: u64) {
    // Split before multiplying so even long elapsed times cannot overflow.
    ticks := (nanoseconds / 1000000000) * 60;

    m.timer_units += (nanoseconds % 1000000000) * 60;
    ticks += m.timer_units / 1000000000;
    m.timer_units %= 1000000000;

    if ticks > 0 {
        m.delay = cast(u8) (cast(u64) m.delay - min(cast(u64) m.delay, ticks));
        m.sound = cast(u8) (cast(u64) m.sound - min(cast(u64) m.sound, ticks));
        m.draw_blocked = false;
    }
}

set_key :: (m: *Machine, key: int, down: bool) {
    if key < 0 || key > 15 return;

    if m.waiting_key >= 0 {
        if down && !m.keys[key] m.released_key = key;

        if !down && m.keys[key] && m.released_key == key {
            m.v[m.waiting_key] = cast(u8) key;
            m.waiting_key = -1;
            m.released_key = -1;
        }
    }

    m.keys[key] = down;
}

// Runs up to cycles instructions; blocked key/draw waits consume no instructions.
step :: (m: *Machine, cycles: int) -> int {
    executed := 0;
    while executed < cycles && !m.halted && m.waiting_key < 0 && !m.draw_blocked {
        if m.pc < 0 || m.pc > 4094 {
            fail(m, "instruction outside memory");
            break;
        }

        op := ((cast(int) m.memory[m.pc]) << 8) | m.memory[m.pc + 1];

        m.pc += 2;
        execute(m, op);
        executed += 1;
    }

    return executed;
}

execute :: (m: *Machine, op: int) {
    x := (op >> 8) & 15;
    y := (op >> 4) & 15;
    n := op & 15;
    nn := op & 255;
    addr := op & 4095;
    a := cast(int) m.v[x];
    b := cast(int) m.v[y];

    if op >> 12 == {
        case 0;
            if op == 0x00E0 {
                for *m.pixels it.* = 0;
            } else if op == 0x00EE {
                if m.sp == 0 {
                    fail(m, "stack underflow");
                    return;
                }

                m.sp -= 1;
                m.pc = m.stack[m.sp];
            } else if op == 0x00FD m.halted = true;
            else if op == 0x00FE || op == 0x00FF {
                m.high = op == 0x00FF;
                for *m.pixels it.* = 0;
            } else if op == 0x00FB scroll(m, 4, 0);
            else if op == 0x00FC scroll(m, -4, 0);
            else if (op & 0xFFF0) == 0x00C0 scroll(m, 0, n);
            else {
                fail(m, "unsupported system opcode");
            }
        case 1; m.pc = addr;
        case 2;
            if m.sp == 16 {
                fail(m, "stack overflow");
                return;
            }

            m.stack[m.sp] = m.pc;
            m.sp += 1;
            m.pc = addr;
        case 3; if a == nn m.pc += 2;
        case 4; if a != nn m.pc += 2;
        case 5;
            if n != 0 {
                fail(m, "invalid equality opcode");
                return;
            }

            if a == b m.pc += 2;
        case 6; m.v[x] = cast(u8) nn;
        case 7; m.v[x] = cast(u8) ((a + nn) & 255);
        case 8; arithmetic(m, x, y, n);
        case 9;
            if n != 0 {
                fail(m, "invalid inequality opcode");
                return;
            }

            if a != b m.pc += 2;
        case 10; m.i = addr;
        case 11; m.pc = addr + m.v[ifx m.quirks.jump_vx then x else 0];
        case 12; m.v[x] = cast(u8) (random_get(*m.rng) & cast(u32) nn);
        case 13; draw(m, a, b, n);
        case 14;
            if nn != 0x9E && nn != 0xA1 {
                fail(m, "invalid key opcode");
                return;
            }

            down := a < 16 && m.keys[a];
            if (nn == 0x9E && down) || (nn == 0xA1 && !down) m.pc += 2;
        case 15; special(m, x, nn);
    }
}

arithmetic :: (m: *Machine, x: int, y: int, n: int) {
    a := cast(int) m.v[x];
    b := cast(int) m.v[y];
    flag := -1;
    result := a;

    if n == {
        case 0; result = b;
        case 1;
            result = a | b;
            if m.quirks.clear_vf flag = 0;
        case 2;
            result = a & b;
            if m.quirks.clear_vf flag = 0;
        case 3;
            result = a ^ b;
            if m.quirks.clear_vf flag = 0;
        case 4;
            result = a + b;
            flag = ifx result > 255 then 1 else 0;
        case 5;
            result = a - b;
            flag = ifx a >= b then 1 else 0;
        case 6;
            value := ifx m.quirks.shift_vy then b else a;

            result = value >> 1;
            flag = value & 1;
        case 7;
            result = b - a;
            flag = ifx b >= a then 1 else 0;
        case 14;
            value := ifx m.quirks.shift_vy then b else a;

            result = value << 1;
            flag = value >> 7;
        case;
            fail(m, "invalid arithmetic opcode");
            return;
    }

    m.v[x] = cast(u8) (result & 255);
    if flag >= 0 m.v[15] = cast(u8) flag;
}

memory_range :: (m: *Machine, count: int) -> bool {
    if m.i < 0 || m.i > 4096 - count return fail(m, "data outside memory");
    return true;
}

special :: (m: *Machine, x: int, nn: int) {
    value := cast(int) m.v[x];

    if nn == {
        case 0x07; m.v[x] = m.delay;
        case 0x0A;
            m.waiting_key = x;
            m.released_key = -1;
        case 0x15; m.delay = m.v[x];
        case 0x18; m.sound = m.v[x];
        case 0x1E; m.i = (m.i + value) & 4095;
        case 0x29; m.i = 0x50 + (value & 15) * 5;
        case 0x30; m.i = 0xA0 + (value & 15) * 10;
        case 0x33;
            if !memory_range(m, 3) return;

            m.memory[m.i] = cast(u8) (value / 100);
            m.memory[m.i + 1] = cast(u8) (value / 10 % 10);
            m.memory[m.i + 2] = cast(u8) (value % 10);
        case 0x55;
            if !memory_range(m, x + 1) return;
            for 0..x m.memory[m.i + it] = m.v[it];
            if m.quirks.increment_i m.i += x + 1;
        case 0x65;
            if !memory_range(m, x + 1) return;
            for 0..x m.v[it] = m.memory[m.i + it];
            if m.quirks.increment_i m.i += x + 1;
        case 0x75;
            if x > 7 {
                fail(m, "only eight RPL flags");
                return;
            }

            memcpy(m.rpl.data, m.v.data, x + 1);
        case 0x85;
            if x > 7 {
                fail(m, "only eight RPL flags");
                return;
            }

            memcpy(m.v.data, m.rpl.data, x + 1);
        case; fail(m, "unsupported F opcode");
    }
}

scroll :: (m: *Machine, dx: int, dy: int) {
    // Traversal follows the move, so the source is never overwritten before being read.
    for row: 0..height(m) - 1 {
        y := height(m) - 1 - row;

        for col: 0..width(m) - 1 {
            x := ifx dx > 0 then width(m) - 1 - col else col;
            sx := x - dx;
            sy := y - dy;
            m.pixels[y * 128 + x] = ifx sx >= 0 && sx < width(m) && sy >= 0 then m.pixels[sy * 128 + sx] else 0;
        }
    }
}

draw :: (m: *Machine, px: int, py: int, n: int) {
    rows := ifx n == 0 then 16 else n;
    cols := ifx n == 0 then 16 else 8;

    if !memory_range(m, rows * cols / 8) return;

    x0 := px % width(m);
    y0 := py % height(m);

    m.v[15] = 0;

    for row: 0..rows - 1 for col : 0..cols - 1 {
        if (m.memory[m.i + row * cols / 8 + col / 8] & (0x80 >> (col % 8))) == 0 continue;

        x := x0 + col;
        y := y0 + row;

        if m.quirks.clip && (x >= width(m) || y >= height(m)) continue;

        at := (y % height(m)) * 128 + (x % width(m));
        if m.pixels[at] != 0 m.v[15] = 1;

        m.pixels[at] ^= 1;
    }

    m.draw_blocked = m.quirks.display_wait;
}

// Hash all framebuffer cells, including the zeroed inactive region, on any CPU.
frame_hash :: (m: *Machine) -> u64, u64 {
    digest := M.hash_bytes(m.pixels);
    return digest[0], digest[1];
}
