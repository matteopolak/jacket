//! The portable lowering: plain LLVM IR that works on every target and is the reference for
//! what the target-specific backends must compute.

use super::{Cx, R};
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::module::Linkage;
use inkwell::types::VectorType;
use inkwell::values::{
    BasicValueEnum, FunctionValue, IntValue, PointerValue, ValueKind, VectorValue,
};
use inkwell::{AddressSpace, IntPredicate};

/// `ByteLookup` one lane at a time (`extractelement`/`select`/`insertelement`).
pub fn byte_lookup<'ctx>(
    cx: &Cx<'_, 'ctx>,
    width: u8,
    dst: PointerValue<'ctx>,
    table: VectorValue<'ctx>,
    idx: PointerValue<'ctx>,
) -> R<bool> {
    let b = cx.builder;
    let i8t = cx.ctx.i8_type();
    let i32t = cx.ctx.i32_type();
    let zero = i8t.const_zero();
    cx.map_blocks(width, 16, idx, dst, |indices| {
        let mut out = i8t.vec_type(16).get_undef();
        for lane in 0..16u64 {
            let at = i32t.const_int(lane, false);
            let i = b.build_extract_element(indices, at, "")?.into_int_value();
            let in_range =
                b.build_int_compare(IntPredicate::ULT, i, i8t.const_int(16, false), "")?;
            let sel = b.build_and(i, i8t.const_int(15, false), "")?;
            let sel = b.build_int_z_extend(sel, i32t, "")?;
            let v = b.build_extract_element(table, sel, "")?.into_int_value();
            let v = b.build_select(in_range, v, zero, "")?.into_int_value();
            out = b.build_insert_element(out, v, at, "")?;
        }
        Ok(BasicValueEnum::from(out))
    })?;
    Ok(true)
}

/// `SignMask` as a vector compare and a bitcast of the `<N x i1>` to `iN`.
pub fn sign_mask<'ctx>(cx: &Cx<'_, 'ctx>, width: u8, src: PointerValue<'ctx>) -> R<IntValue<'ctx>> {
    let b = cx.builder;
    let v = cx.load_bytes(src, u32::from(width))?;
    let zero = cx.ctx.i8_type().vec_type(u32::from(width)).const_zero();
    let neg = b.build_int_compare(IntPredicate::SLT, v, zero, "")?;
    let bits = cx
        .ctx
        .custom_width_int_type(std::num::NonZeroU32::new(u32::from(width)).ok_or("zero width")?)?;
    let packed = b.build_bit_cast(neg, bits, "")?.into_int_value();
    Ok(b.build_int_z_extend_or_bit_cast(packed, cx.ctx.i64_type(), "")?)
}

/// `StringCompare` as a call to `jaic.string_compare.<ctl>.<i|e>`, made once per module from
/// lane-wide vector compares. One helper per control byte keeps each use a call: expanded at
/// every use, the element compares made blocks long enough for LLVM's machine passes (which
/// grow with the square of a block's length) to take minutes. Returns `(result, b_ends, a_ends)`
/// as `ir::string_compare` defines them.
pub fn string_compare<'ctx>(
    cx: &Cx<'_, 'ctx>,
    (ctl, explicit): (u8, bool),
    (a, b): (PointerValue<'ctx>, PointerValue<'ctx>),
    (len_a, len_b): (IntValue<'ctx>, IntValue<'ctx>),
) -> R<[IntValue<'ctx>; 3]> {
    let name = format!(
        "jaic.string_compare.{ctl}.{}",
        if explicit {
            'e'
        } else {
            'i'
        }
    );
    let helper = match cx.module.get_function(&name) {
        Some(f) => f,
        None => string_compare_helper(cx, &name, ctl, explicit)?,
    };
    let call = cx.builder.build_call(
        helper,
        &[a.into(), b.into(), len_a.into(), len_b.into()],
        "",
    )?;
    let ValueKind::Basic(result) = call.try_as_basic_value() else {
        return Err("the string compare helper returned no value".into());
    };
    let result = result.into_struct_value();
    let field = |i| -> R<IntValue<'ctx>> {
        Ok(cx
            .builder
            .build_extract_value(result, i, "")?
            .into_int_value())
    };
    Ok([field(0)?, field(1)?, field(2)?])
}

/// The body of one string compare helper; see `ir::string_compare` for what each mode computes.
fn string_compare_helper<'ctx>(
    outer: &Cx<'_, 'ctx>,
    name: &str,
    ctl: u8,
    explicit: bool,
) -> R<FunctionValue<'ctx>> {
    let ctx = outer.ctx;
    let (i8t, i32t) = (ctx.i8_type(), ctx.i32_type());
    let ptr = ctx.ptr_type(AddressSpace::default());
    let ret = ctx.struct_type(&[i32t.into(), i8t.into(), i8t.into()], false);
    let f = outer.module.add_function(
        name,
        ret.fn_type(&[ptr.into(), ptr.into(), i32t.into(), i32t.into()], false),
        Some(Linkage::Internal),
    );
    for attr in ["nounwind", "willreturn"] {
        let kind = Attribute::get_named_enum_kind_id(attr);
        f.add_attribute(AttributeLoc::Function, ctx.create_enum_attribute(kind, 0));
    }
    let builder = ctx.create_builder();
    builder.position_at_end(ctx.append_basic_block(f, "entry"));
    let cx = Cx {
        builder: &builder,
        ..*outer
    };
    let b = cx.builder;
    let param = |i| f.get_nth_param(i).ok_or("missing helper parameter");
    let words = ctl & 1 != 0;
    let signed = ctl & 2 != 0;
    let n: u32 = if words {
        8
    } else {
        16
    };
    let lane_ty = if words {
        ctx.i16_type()
    } else {
        i8t
    };
    let vt = lane_ty.vec_type(n);
    let bools = ctx.bool_type().vec_type(n);
    let bits_ty = ctx.custom_width_int_type(std::num::NonZeroU32::new(n).ok_or("no lanes")?)?;
    let load = |p: BasicValueEnum<'ctx>| -> R<VectorValue<'ctx>> {
        let v = cx.load_bytes(p.into_pointer_value(), 16)?;
        Ok(b.build_bit_cast(v, vt, "")?.into_vector_value())
    };
    let va_ = load(param(0)?)?;
    let vb_ = load(param(1)?)?;
    let k = |v: u64| i32t.const_int(v, false);
    let all = (1u64 << n) - 1;
    // Which elements belong to each string, as bits of an i32 and as a vector of booleans.
    let valid =
        |v: VectorValue<'ctx>, len: IntValue<'ctx>| -> R<(IntValue<'ctx>, VectorValue<'ctx>)> {
            let mask = if explicit {
                let neg = b.build_int_compare(IntPredicate::SLT, len, k(0), "")?;
                let negated = b.build_int_sub(k(0), len, "")?;
                let abs = b.build_select(neg, negated, len, "")?.into_int_value();
                let short = b.build_int_compare(IntPredicate::ULT, abs, k(u64::from(n)), "")?;
                let count = b
                    .build_select(short, abs, k(u64::from(n)), "")?
                    .into_int_value();
                let top = b.build_left_shift(k(1), count, "")?;
                b.build_int_sub(top, k(1), "")?
            } else {
                // Below the lowest zero element (or all of them): (lowest set bit of zeros) - 1.
                let zeros = b.build_int_compare(IntPredicate::EQ, v, vt.const_zero(), "")?;
                let zeros = b.build_bit_cast(zeros, bits_ty, "")?.into_int_value();
                let zeros = b.build_int_z_extend(zeros, i32t, "")?;
                let guarded = b.build_or(zeros, k(1 << n), "")?;
                let negated = b.build_int_sub(k(0), guarded, "")?;
                let lowest = b.build_and(guarded, negated, "")?;
                b.build_int_sub(lowest, k(1), "")?
            };
            let narrow = b.build_int_truncate(mask, bits_ty, "")?;
            let vector = b.build_bit_cast(narrow, bools, "")?.into_vector_value();
            Ok((mask, vector))
        };
    let (ma, valid_a) = valid(va_, param(2)?.into_int_value())?;
    let (mb, valid_b) = valid(vb_, param(3)?.into_int_value())?;
    let splat = |v: VectorValue<'ctx>, lane: u32| -> R<VectorValue<'ctx>> {
        let mask = VectorType::const_vector(&vec![k(u64::from(lane)); n as usize]);
        Ok(b.build_shuffle_vector(v, v, mask, "")?)
    };
    let lane_valid = |lane: u32| -> R<IntValue<'ctx>> {
        Ok(b.build_extract_element(valid_a, k(u64::from(lane)), "")?
            .into_int_value())
    };
    let none = bools.const_zero();
    let every = VectorType::const_vector(&vec![ctx.bool_type().const_int(1, false); n as usize]);
    let result = match (ctl >> 2) & 3 {
        // Equal any: some valid element of `a` equals this element of `b`.
        0 => {
            let mut acc = none;
            for j in 0..n {
                let eq = b.build_int_compare(IntPredicate::EQ, vb_, splat(va_, j)?, "")?;
                let hit = b
                    .build_select(lane_valid(j)?, eq, none, "")?
                    .into_vector_value();
                acc = b.build_or(acc, hit, "")?;
            }
            b.build_and(acc, valid_b, "")?
        }
        // Ranges: `a` holds (low, high) pairs; this element of `b` lies in one of them.
        1 => {
            let (ge, le) = if signed {
                (IntPredicate::SGE, IntPredicate::SLE)
            } else {
                (IntPredicate::UGE, IntPredicate::ULE)
            };
            let mut acc = none;
            for p in 0..n / 2 {
                let above = b.build_int_compare(ge, vb_, splat(va_, 2 * p)?, "")?;
                let below = b.build_int_compare(le, vb_, splat(va_, 2 * p + 1)?, "")?;
                let inside = b.build_and(above, below, "")?;
                let pair = b.build_and(lane_valid(2 * p)?, lane_valid(2 * p + 1)?, "")?;
                let hit = b.build_select(pair, inside, none, "")?.into_vector_value();
                acc = b.build_or(acc, hit, "")?;
            }
            b.build_and(acc, valid_b, "")?
        }
        // Equal each: pairwise; two missing elements match, one missing does not.
        2 => {
            let eq = b.build_int_compare(IntPredicate::EQ, va_, vb_, "")?;
            let both = b.build_and(valid_a, valid_b, "")?;
            let hit = b.build_and(eq, both, "")?;
            let either = b.build_or(valid_a, valid_b, "")?;
            let neither = b.build_xor(either, every, "")?;
            b.build_or(hit, neither, "")?
        }
        // Equal ordered: `a` occurs in `b` starting at this element (a missing `a` element
        // matches). Term j compares element j of `a` with element i + j of `b`.
        _ => {
            let mut acc = every;
            for j in 0..n {
                let shift: Vec<IntValue> = (0..n)
                    .map(|i| {
                        k(u64::from(if i + j < n {
                            i + j
                        } else {
                            i
                        }))
                    })
                    .collect();
                let shift = VectorType::const_vector(&shift);
                let shifted = b.build_shuffle_vector(vb_, vb_, shift, "")?;
                let shifted_valid = b.build_shuffle_vector(valid_b, valid_b, shift, "")?;
                let eq = b.build_int_compare(IntPredicate::EQ, shifted, splat(va_, j)?, "")?;
                let hit = b.build_and(eq, shifted_valid, "")?;
                let term = b
                    .build_select(lane_valid(j)?, hit, every, "")?
                    .into_vector_value();
                // Elements whose term j would pass the end of `b` do not need it.
                let past: Vec<_> = (0..n)
                    .map(|i| ctx.bool_type().const_int(u64::from(i + j >= n), false))
                    .collect();
                let term = b.build_or(term, VectorType::const_vector(&past), "")?;
                acc = b.build_and(acc, term, "")?;
            }
            acc
        }
    };
    let result = b.build_bit_cast(result, bits_ty, "")?.into_int_value();
    let result = b.build_int_z_extend(result, i32t, "")?;
    // Polarity: negative inverts, masked negative inverts only the valid elements of `b`.
    let result = match (ctl >> 4) & 3 {
        1 => b.build_xor(result, k(all), "")?,
        3 => b.build_xor(result, mb, "")?,
        _ => result,
    };
    let ends = |mask: IntValue<'ctx>| -> R<IntValue<'ctx>> {
        let last = b.build_and(mask, k(1 << (n - 1)), "")?;
        let ended = b.build_int_compare(IntPredicate::EQ, last, k(0), "")?;
        Ok(b.build_int_z_extend(ended, i8t, "")?)
    };
    let (b_ends, a_ends) = (ends(mb)?, ends(ma)?);
    let mut out = ret.get_undef();
    for (i, v) in [result, b_ends, a_ends].into_iter().enumerate() {
        out = b
            .build_insert_value(out, v, i as u32, "")?
            .into_struct_value();
    }
    b.build_return(Some(&out))?;
    Ok(f)
}
