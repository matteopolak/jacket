use super::*;
use jai_types::{ScalarLayout, TypeRegistry};

fn fixture() -> (TypeRegistry, TypeId, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(element, 3).unwrap();
    let slice = types.slice(element).unwrap();
    (types, element, array, slice)
}
fn zero(ty: TypeId) -> ConstantValue {
    ConstantValue {
        ty,
        kind: ConstantKind::Zero,
    }
}
fn batch() -> CapturedSliceBatch {
    CapturedSliceBatch::new(
        b"retained-source-origin".to_vec(),
        2,
        ConstantSliceLimits::default(),
    )
    .unwrap()
}

#[test]
fn literal_views_retain_full_typed_storage_and_share_subranges() {
    let (types, element, array, slice) = fixture();
    let limits = ConstantSliceLimits::default();
    let backing =
        ConstantSliceBacking::literal(&types, array, vec![zero(element); 3], limits).unwrap();
    assert_eq!(backing.access(), ConstantSliceAccess::ReadOnly);
    assert_eq!(
        backing
            .selected_extent(&types, &LayoutPolicy::lp64())
            .unwrap()
            .bytes,
        24
    );
    let full = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(Arc::clone(&backing)),
        0,
        3,
        limits,
    )
    .unwrap();
    let sub = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(Arc::clone(&backing)),
        1,
        2,
        limits,
    )
    .unwrap();
    let copy = sub
        .checked_clone(&types, &LayoutPolicy::lp64(), limits)
        .unwrap();
    assert!(Arc::ptr_eq(
        full.backing().unwrap(),
        copy.backing().unwrap()
    ));
    assert_eq!((copy.start(), copy.count()), (1, 2));
    assert_eq!(backing.slots().len(), 3);
}

#[test]
fn source_storage_class_and_canonical_element_identity_are_checked() {
    let (mut types, element, array, slice) = fixture();
    let limits = ConstantSliceLimits::default();
    assert!(matches!(
        ConstantSliceBacking::literal(&types, array, vec![zero(element)], limits),
        Err(ConstantSliceError::InvalidLiteral(_))
    ));
    let u8_ty = types.scalar(ScalarType::Int(IntegerType::U8));
    let bytes_slice = types.slice(u8_ty).unwrap();
    let bytes =
        ConstantSliceBacking::literal(&types, types.string(), vec![zero(u8_ty)], limits).unwrap();
    assert!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            bytes_slice,
            Some(Arc::clone(&bytes)),
            0,
            1,
            limits
        )
        .is_ok()
    );
    assert!(matches!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(bytes),
            0,
            1,
            limits
        ),
        Err(ConstantSliceError::ElementType { .. })
    ));
    let other = TypeRegistry::new();
    assert!(ConstantSliceBacking::literal(&other, array, vec![zero(element); 3], limits).is_err());
}

#[test]
fn captured_identity_distinguishes_equal_roots_but_receipt_is_stable() {
    let (types, element, _, _) = fixture();
    let limits = ConstantSliceLimits::default();
    let mut first_batch = batch();
    let a = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element))],
        ConstantSliceAccess::Mutable,
        &mut first_batch,
        limits,
    )
    .unwrap();
    let b = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element))],
        ConstantSliceAccess::Mutable,
        &mut first_batch,
        limits,
    )
    .unwrap();
    let c = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element))],
        ConstantSliceAccess::Mutable,
        &mut batch(),
        limits,
    )
    .unwrap();
    let (
        ConstantSliceOrigin::Captured(a),
        ConstantSliceOrigin::Captured(b),
        ConstantSliceOrigin::Captured(c),
    ) = (a.origin(), b.origin(), c.origin())
    else {
        panic!("actual captured identities")
    };
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.receipt(), c.receipt());
    assert_eq!(a.receipt().backing_ordinal(), 0);
    assert_eq!(b.receipt().backing_ordinal(), 1);
}

#[test]
fn empty_captured_nonnull_one_past_is_not_replaced_by_null() {
    let (types, element, _, slice) = fixture();
    let limits = ConstantSliceLimits::default();
    let backing = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element)); 3],
        ConstantSliceAccess::Mutable,
        &mut batch(),
        limits,
    )
    .unwrap();
    let view = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(backing),
        3,
        0,
        limits,
    )
    .unwrap();
    assert!(view.backing().is_some());
    assert_eq!(view.start(), 3);
    let null =
        ConstantSlice::new(&types, &LayoutPolicy::lp64(), slice, None, 0, 0, limits).unwrap();
    assert!(null.backing().is_none());
    assert!(ConstantSlice::new(&types, &LayoutPolicy::lp64(), slice, None, 1, 0, limits).is_err());
}

#[test]
fn uninitialized_capacity_and_visible_holes_are_preserved_without_reads() {
    let (types, element, _, slice) = fixture();
    let limits = ConstantSliceLimits::default();
    let backing = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element)), Some(zero(element)), None, None],
        ConstantSliceAccess::Mutable,
        &mut batch(),
        limits,
    )
    .unwrap();
    assert_eq!(backing.slots().len(), 4);
    let view = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(Arc::clone(&backing)),
        0,
        2,
        limits,
    )
    .unwrap();
    assert_eq!(
        view.backing().unwrap().access(),
        ConstantSliceAccess::Mutable
    );
    let holes = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(Arc::clone(&backing)),
        1,
        2,
        limits,
    )
    .unwrap();
    assert!(holes.backing().unwrap().slots()[2].is_none());
    assert!(matches!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(backing),
            u64::MAX,
            2,
            limits
        ),
        Err(ConstantSliceError::Range { .. })
    ));
    assert!(matches!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            None,
            0,
            u64::MAX,
            limits
        ),
        Err(ConstantSliceError::SignedCount(_))
    ));
}

#[test]
fn target_extent_uses_selected_pointer_domain_and_actual_element_stride() {
    let (mut types, byte, _, _) = fixture();
    let giant = types.fixed_array(byte, 400_000_000).unwrap();
    let slice = types.slice(giant).unwrap();
    let limits = ConstantSliceLimits::default();
    let backing = ConstantSliceBacking::captured(
        &types,
        giant,
        vec![Some(zero(giant)); 2],
        ConstantSliceAccess::ReadOnly,
        &mut batch(),
        limits,
    )
    .unwrap();
    let ilp32 = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    assert!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(Arc::clone(&backing)),
            0,
            2,
            limits
        )
        .is_ok()
    );
    assert!(matches!(
        ConstantSlice::new(&types, &ilp32, slice, Some(backing), 0, 2, limits),
        Err(ConstantSliceError::TargetExtent)
    ));
}

#[test]
fn rejected_deep_values_are_disposed_iteratively_before_retention() {
    let (types, element, _, _) = fixture();
    let mut value = zero(element);
    for _ in 0..20_000 {
        value = ConstantValue {
            ty: element,
            kind: ConstantKind::Array(vec![value]),
        };
    }
    assert!(matches!(
        ConstantSliceBacking::captured(
            &types,
            element,
            vec![Some(value)],
            ConstantSliceAccess::ReadOnly,
            &mut batch(),
            ConstantSliceLimits::default()
        ),
        Err(ConstantSliceError::Limit("value depth"))
    ));
}

#[test]
fn limits_reject_before_identity_publication_and_clone() {
    let (types, element, _, slice) = fixture();
    let limits = ConstantSliceLimits::default();
    let mut publish = batch();
    assert!(matches!(
        ConstantSliceBacking::captured(
            &types,
            element,
            vec![None],
            ConstantSliceAccess::ReadOnly,
            &mut publish,
            ConstantSliceLimits {
                value_depth: 0,
                ..limits
            }
        ),
        Err(ConstantSliceError::Limit("value depth"))
    ));
    let tiny = ConstantSliceLimits {
        value_nodes: 1,
        ..limits
    };
    assert!(
        ConstantSliceBacking::captured(
            &types,
            element,
            vec![Some(zero(element))],
            ConstantSliceAccess::ReadOnly,
            &mut publish,
            tiny
        )
        .is_err()
    );
    let backing = ConstantSliceBacking::captured(
        &types,
        element,
        vec![Some(zero(element))],
        ConstantSliceAccess::ReadOnly,
        &mut publish,
        limits,
    )
    .unwrap();
    let ConstantSliceOrigin::Captured(identity) = backing.origin() else {
        panic!("captured")
    };
    assert_eq!(identity.receipt().backing_ordinal(), 0);
    let view = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(backing),
        0,
        1,
        limits,
    )
    .unwrap();
    assert!(
        view.checked_clone(&types, &LayoutPolicy::lp64(), tiny)
            .is_err()
    );
    assert!(CapturedSliceBatch::new(vec![], 0, limits).is_err());
    assert!(
        CapturedSliceBatch::new(
            vec![0; 3],
            0,
            ConstantSliceLimits {
                receipt_bytes: 2,
                ..limits
            }
        )
        .is_err()
    );
}

#[test]
fn literal_zero_stride_capacity_is_authoritative_and_range_still_checked() {
    let (mut types, byte, _, _) = fixture();
    let zst = types.fixed_array(byte, 0).unwrap();
    let storage = types.fixed_array(zst, 3).unwrap();
    let slice = types.slice(zst).unwrap();
    let limits = ConstantSliceLimits::default();
    let backing =
        ConstantSliceBacking::literal(&types, storage, vec![zero(zst); 3], limits).unwrap();
    assert_eq!(backing.capacity(), 3);
    assert_eq!(
        backing
            .selected_extent(&types, &LayoutPolicy::lp64())
            .unwrap()
            .bytes,
        0
    );
    assert!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(Arc::clone(&backing)),
            0,
            3,
            limits
        )
        .is_ok()
    );
    assert!(matches!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(backing),
            0,
            4,
            limits
        ),
        Err(ConstantSliceError::Range { .. })
    ));
}

#[test]
fn empty_literal_uses_null_and_captured_holes_do_not_require_value_reads() {
    let (mut types, element, _, slice) = fixture();
    let empty_array = types.fixed_array(element, 0).unwrap();
    let limits = ConstantSliceLimits::default();
    let literal = ConstantSliceBacking::literal(&types, empty_array, vec![], limits).unwrap();
    assert!(matches!(
        ConstantSlice::new(
            &types,
            &LayoutPolicy::lp64(),
            slice,
            Some(literal),
            0,
            0,
            limits
        ),
        Err(ConstantSliceError::InvalidLiteral(_))
    ));
    assert!(ConstantSlice::new(&types, &LayoutPolicy::lp64(), slice, None, 0, 0, limits).is_ok());
    let captured = ConstantSliceBacking::captured(
        &types,
        element,
        vec![None; 3],
        ConstantSliceAccess::Mutable,
        &mut batch(),
        limits,
    )
    .unwrap();
    let view = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(captured),
        0,
        3,
        limits,
    )
    .unwrap();
    assert!(view.backing().unwrap().slots().iter().all(Option::is_none));
}

#[test]
fn owned_string_bytes_are_budgeted_before_retention_and_clone() {
    let mut types = TypeRegistry::new();
    let string = types.string();
    let slice = types.slice(string).unwrap();
    let limits = ConstantSliceLimits::default();
    let tiny = ConstantSliceLimits {
        owned_bytes: 5,
        ..limits
    };
    let values = vec![
        Some(ConstantValue {
            ty: string,
            kind: ConstantKind::StringBytes(vec![b'a'; 3]),
        }),
        Some(ConstantValue {
            ty: string,
            kind: ConstantKind::StringBytes(vec![b'b'; 3]),
        }),
    ];
    assert!(matches!(
        ConstantSliceBacking::captured(
            &types,
            string,
            values.clone(),
            ConstantSliceAccess::ReadOnly,
            &mut batch(),
            tiny
        ),
        Err(ConstantSliceError::Limit("owned bytes"))
    ));
    let backing = ConstantSliceBacking::captured(
        &types,
        string,
        values,
        ConstantSliceAccess::ReadOnly,
        &mut batch(),
        limits,
    )
    .unwrap();
    assert_eq!(backing.owned_bytes(), 6);
    assert_eq!(backing.work(), backing.cells() + 6);
    let view = ConstantSlice::new(
        &types,
        &LayoutPolicy::lp64(),
        slice,
        Some(backing),
        0,
        2,
        limits,
    )
    .unwrap();
    assert!(matches!(
        view.checked_clone(&types, &LayoutPolicy::lp64(), tiny),
        Err(ConstantSliceError::Limit("owned bytes"))
    ));
}
