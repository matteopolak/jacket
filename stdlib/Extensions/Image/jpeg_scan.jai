// Scan decoding: the SOS header, the MCU loop with restart handling, and the per-block
// decoders for sequential and progressive scans.
//
// Sequential scans transform each block into its component plane as soon as it is decoded.
// Progressive scans accumulate quantized coefficients (natural order, undequantized) in
// Jpeg_Component.coefs and transform everything once the last scan is in.

JPEG_SEQUENTIAL :: 0;
JPEG_DC_FIRST :: 1;
JPEG_DC_REFINE :: 2;
JPEG_AC_FIRST :: 3;
JPEG_AC_REFINE :: 4;

Jpeg_Scan_Block :: struct {
    comp: s32;
    scale_x: s32;
    scale_y: s32;
    off_x: s32;
    off_y: s32;
    dc: *Jpeg_Huffman;
    ac: *Jpeg_Huffman;
}

jpeg_read_scan :: (d: *Jpeg_Decoder, segment: *u8, size: s64) -> string {
    if !d.frame_seen return "JPEG scan before the frame header";

    count := cast(s32) segment[0];
    if size < 1 || count < 1 || count > d.ncomp || size != 1 + 2 * count + 3 return "bad scan header";

    selected: [4] s32;
    dc_ids: [4] s32;
    ac_ids: [4] s32;
    used: u32 = 0;

    for 0..count - 1 {
        id := cast(s32) segment[1 + it * 2];
        found: s32 = -1;

        for c: 0..d.ncomp - 1 {
            if d.comps[c].id == id {
                found = c;
                break;
            }
        }

        if found < 0 return "JPEG scan names a component the frame does not have";
        if used & ((cast(u32) 1) << cast(u32) found) return "bad scan header";

        used |= (cast(u32) 1) << cast(u32) found;
        selected[it] = found;
        dc_ids[it] = segment[2 + it * 2] >> 4;
        ac_ids[it] = segment[2 + it * 2] & 15;
        if dc_ids[it] > 3 || ac_ids[it] > 3 return "bad scan header";
    }

    ss := cast(s32) segment[1 + count * 2];
    se := cast(s32) segment[2 + count * 2];
    ah := cast(s32) (segment[3 + count * 2] >> 4);
    al := cast(s32) (segment[3 + count * 2] & 15);

    mode: s32 = JPEG_SEQUENTIAL;

    if d.progressive {
        if ss > se || se > 63 || al > 13 || ah > 13 return "bad progressive scan header";

        if ss == 0 {
            if se != 0 return "bad progressive scan header";
            mode = ifx ah == 0 then JPEG_DC_FIRST else JPEG_DC_REFINE;
        } else {
            if count != 1 return "bad progressive scan header";
            mode = ifx ah == 0 then JPEG_AC_FIRST else JPEG_AC_REFINE;
        }
    }

    blocks: [10] Jpeg_Scan_Block;
    block_count: s32 = 0;

    for 0..count - 1 {
        comp := *d.comps[selected[it]];
        if !comp.quant_taken {
            if !d.quant_defined[comp.quant_id] return "JPEG scan uses an undefined quantization table";

            for i: 0..63 {
                comp.quant[i] = d.quant_tables[comp.quant_id][i];
                comp.quant_natural[JPEG_ZIGZAG[i]] = comp.quant[i];
            }

            comp.quant_taken = true;
        }

        dc: *Jpeg_Huffman;
        ac: *Jpeg_Huffman;

        if mode == JPEG_SEQUENTIAL || mode == JPEG_DC_FIRST {
            if !d.dc_tables[dc_ids[it]].defined return "JPEG scan uses an undefined Huffman table";
            dc = *d.dc_tables[dc_ids[it]];
        }

        if mode == JPEG_SEQUENTIAL || mode == JPEG_AC_FIRST || mode == JPEG_AC_REFINE {
            if !d.ac_tables[ac_ids[it]].defined return "JPEG scan uses an undefined Huffman table";
            ac = *d.ac_tables[ac_ids[it]];
        }

        if count == 1 {
            blocks[0] = .{ selected[it], 1, 1, 0, 0, dc, ac };
            block_count = 1;
        } else {
            for y: 0..comp.v - 1 for x : 0..comp.h - 1 {
                if block_count >= 10 return "too many blocks in a JPEG MCU";
                blocks[block_count] = .{ selected[it], comp.h, comp.v, x, y, dc, ac };
                block_count += 1;
            }
        }
    }

    mcus_w := d.mcus_x;
    mcus_h := d.mcus_y;

    if count == 1 {
        mcus_w = d.comps[selected[0]].blocks_w;
        mcus_h = d.comps[selected[0]].blocks_h;
    }

    d.scans += 1;
    jpeg_decode_scan(d, *blocks, block_count, mcus_w, mcus_h, mode, ss, se, al);

    return "";
}

jpeg_decode_scan :: (d: *Jpeg_Decoder, blocks: *[10] Jpeg_Scan_Block, block_count: s32, mcus_w: s32, mcus_h: s32, mode: s32, ss: s32, se: s32, al: s32) {
    d.bits = 0;
    d.nbits = 0;
    d.pad = 0;
    d.marker = 0;

    total := cast(s64) mcus_w * mcus_h;
    interval := cast(s64) d.restart_interval;
    block := d.block.data;
    mcu: s64 = 0;
    expected: s32 = 0;

    while mcu < total {
        for 0..d.ncomp - 1 d.comps[it].pred = 0;

        d.eobrun = 0;

        stop := total;
        if interval > 0 && mcu + interval < total stop = mcu + interval;

        while mcu < stop {
            my := mcu / mcus_w;
            mx := mcu - my * mcus_w;

            for i: 0..block_count - 1 {
                sb := *blocks.*[i];
                comp := *d.comps[sb.comp];
                x := cast(s32) mx * sb.scale_x + sb.off_x;
                y := cast(s32) my * sb.scale_y + sb.off_y;

                if mode == JPEG_SEQUENTIAL {
                    last := jpeg_block_sequential(d, comp, block, sb.dc, sb.ac);

                    if x < comp.blocks_w && y < comp.blocks_h {
                        jpeg_idct_block(block, last, comp.plane + cast(s64) y * 8 * comp.stride + cast(s64) x * 8, comp.stride);
                    }

                    if last > 0 memset(block, 0, 256);
                } else {
                    coefs := comp.coefs + (cast(s64) y * comp.alloc_w + x) * 64;

                    if mode == JPEG_DC_FIRST {
                        jpeg_block_dc_first(d, comp, coefs, sb.dc, al);
                    } else if mode == JPEG_DC_REFINE {
                        jpeg_block_dc_refine(d, coefs, al);
                    } else if mode == JPEG_AC_FIRST {
                        jpeg_block_ac_first(d, coefs, sb.ac, ss, se, al);
                    } else {
                        jpeg_block_ac_refine(d, coefs, sb.ac, ss, se, al);
                    }
                }
            }

            mcu += 1;
            if d.pad > d.nbits break; // the entropy data ran out
        }

        if mcu >= total break;
        if interval == 0 break;

        mcu = stop;
        if mcu >= total break;

        skipped: s32;
        if !jpeg_sync_restart(d, *expected, *skipped) break;

        mcu += cast(s64) skipped * interval;
    }
}

jpeg_block_sequential :: (d: *Jpeg_Decoder, comp: *Jpeg_Component, block: *s32, dc: *Jpeg_Huffman, ac: *Jpeg_Huffman) -> s32 #no_abc #no_aoc {
    bits := d.bits;
    nbits := d.nbits;
    pos := d.pos;
    data := d.data;
    end := d.end;
    quant := comp.quant.data;

    jpeg_refill();
    size := jpeg_decode_symbol(dc);
    diff: s32 = 0;

    if size != 0 diff = jpeg_extend(jpeg_get_bits(size), size);

    comp.pred += diff;
    block[0] = comp.pred * quant[0];

    last: s32 = 0;
    k: s32 = 1;

    while k < 64 {
        jpeg_refill();
        entry := ac.ac_fast[bits >> 55];
        if entry != 0 {
            used := cast(s64) (entry & 255);

            bits <<= cast(u64) used;
            nbits -= used;
            run := cast(s32) ((entry >> 8) & 255);
            if run == 255 break;

            k += run;
            if k > 63 break;

            block[JPEG_ZIGZAG[k]] = ((cast,no_check(s32) entry) >> 16) * quant[k];
            last = k;
            k += 1;
        } else {
            symbol := jpeg_decode_symbol(ac);
            run := symbol >> 4;

            size = symbol & 15;
            if size == 0 {
                if run != 15 break;
                k += 16;
                continue;
            }

            k += run;
            if k > 63 break;

            block[JPEG_ZIGZAG[k]] = jpeg_extend(jpeg_get_bits(size), size) * quant[k];
            last = k;
            k += 1;
        }
    }

    d.bits = bits;
    d.nbits = nbits;
    d.pos = pos;

    return last;
}

jpeg_block_dc_first :: (d: *Jpeg_Decoder, comp: *Jpeg_Component, coefs: *s16, dc: *Jpeg_Huffman, al: s32) #no_abc #no_aoc {
    bits := d.bits;
    nbits := d.nbits;
    pos := d.pos;
    data := d.data;
    end := d.end;

    jpeg_refill();
    size := jpeg_decode_symbol(dc);
    diff: s32 = 0;

    if size != 0 diff = jpeg_extend(jpeg_get_bits(size), size);

    comp.pred += diff;
    coefs[0] = cast,trunc(s16) (comp.pred << cast(u32) al);

    d.bits = bits;
    d.nbits = nbits;
    d.pos = pos;
}

jpeg_block_dc_refine :: (d: *Jpeg_Decoder, coefs: *s16, al: s32) #no_abc #no_aoc {
    bits := d.bits;
    nbits := d.nbits;
    pos := d.pos;
    data := d.data;
    end := d.end;

    jpeg_refill();
    if jpeg_get_bits(1) != 0 coefs[0] = cast,trunc(s16) (cast(s32) coefs[0] | (1 << cast(u32) al));

    d.bits = bits;
    d.nbits = nbits;
    d.pos = pos;
}

jpeg_block_ac_first :: (d: *Jpeg_Decoder, coefs: *s16, ac: *Jpeg_Huffman, ss: s32, se: s32, al: s32) #no_abc #no_aoc {
    if d.eobrun > 0 {
        d.eobrun -= 1;
        return;
    }

    bits := d.bits;
    nbits := d.nbits;
    pos := d.pos;
    data := d.data;
    end := d.end;

    k := ss;

    while k <= se {
        jpeg_refill();
        entry := ac.ac_fast[bits >> 55];
        if entry != 0 {
            used := cast(s64) (entry & 255);

            bits <<= cast(u64) used;
            nbits -= used;
            run := cast(s32) ((entry >> 8) & 255);
            if run == 255 break;

            k += run;
            if k > se break;

            coefs[JPEG_ZIGZAG[k]] = cast,trunc(s16) (((cast,no_check(s32) entry) >> 16) << cast(u32) al);
            k += 1;
        } else {
            symbol := jpeg_decode_symbol(ac);
            run := symbol >> 4;
            size := symbol & 15;

            if size == 0 {
                if run < 15 {
                    pending: s32 = 1 << cast(u32) run;
                    if run > 0 pending += jpeg_get_bits(run);
                    d.eobrun = pending - 1;
                    break;
                }

                k += 16;
                continue;
            }

            k += run;
            if k > se break;

            coefs[JPEG_ZIGZAG[k]] = cast,trunc(s16) (jpeg_extend(jpeg_get_bits(size), size) << cast(u32) al);
            k += 1;
        }
    }

    d.bits = bits;
    d.nbits = nbits;
    d.pos = pos;
}

// One more bit of precision for coefficients that are already non-zero, and new +-1 coefficients.
jpeg_block_ac_refine :: (d: *Jpeg_Decoder, coefs: *s16, ac: *Jpeg_Huffman, ss: s32, se: s32, al: s32) #no_abc #no_aoc {
    bits := d.bits;
    nbits := d.nbits;
    pos := d.pos;
    data := d.data;
    end := d.end;

    one: s32 = 1 << cast(u32) al;
    k := ss;

    if d.eobrun == 0 {
        while k <= se {
            jpeg_refill();
            symbol := jpeg_decode_symbol(ac);
            run := symbol >> 4;
            size := symbol & 15;
            value: s32 = 0;

            if size == 0 {
                if run < 15 {
                    pending: s32 = 1 << cast(u32) run;
                    if run > 0 pending += jpeg_get_bits(run);
                    d.eobrun = pending;
                    break;
                }
            } else {
                value = ifx jpeg_get_bits(1) != 0 then one else -one;
            }

            // Skip `run` zero coefficients, refining the non-zero ones passed on the way.
            while k <= se {
                slot := coefs + JPEG_ZIGZAG[k];
                current := cast(s32) slot.*;

                if current != 0 {
                    jpeg_refill();

                    if jpeg_get_bits(1) != 0 && (current & one) == 0 {
                        slot.* = cast,trunc(s16) (ifx current >= 0 then current + one else current - one);
                    }
                } else {
                    if run == 0 break;
                    run -= 1;
                }

                k += 1;
            }

            if value != 0 && k <= se coefs[JPEG_ZIGZAG[k]] = cast,trunc(s16) value;

            k += 1;
        }
    }

    if d.eobrun > 0 {
        while k <= se {
            slot := coefs + JPEG_ZIGZAG[k];
            current := cast(s32) slot.*;

            if current != 0 {
                jpeg_refill();

                if jpeg_get_bits(1) != 0 && (current & one) == 0 {
                    slot.* = cast,trunc(s16) (ifx current >= 0 then current + one else current - one);
                }
            }

            k += 1;
        }

        d.eobrun -= 1;
    }

    d.bits = bits;
    d.nbits = nbits;
    d.pos = pos;
}

// Dequantizes and transforms the accumulated coefficients of a progressive image.
jpeg_finish_progressive :: (d: *Jpeg_Decoder) #no_aoc {
    block := d.block.data;

    for c: 0..d.ncomp - 1 {
        comp := *d.comps[c];
        if !comp.quant_taken continue;

        quant := comp.quant_natural.data;

        for y: 0..comp.blocks_h - 1 for x : 0..comp.blocks_w - 1 {
            coefs := comp.coefs + (cast(s64) y * comp.alloc_w + x) * 64;
            out := comp.plane + cast(s64) y * 8 * comp.stride + cast(s64) x * 8;

            any_ac: s32 = 0;

            for 1..63 any_ac |= coefs[it];

            if any_ac == 0 {
                block[0] = coefs[0] * quant[0];
                jpeg_idct_block(block, 0, out, comp.stride);
            } else {
                for 0..63 block[it] = coefs[it] * quant[it];
                jpeg_idct_block(block, 63, out, comp.stride);
            }
        }
    }
}
