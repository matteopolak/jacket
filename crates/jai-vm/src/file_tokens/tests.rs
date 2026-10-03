use super::*;
use crate::{
    Limits, Value,
    host_effects::{FileRootId, HostPath},
    virtual_files::{FileOpenMode, VirtualFiles},
};
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};
fn fixture() -> (TypeRegistry, TypeId, Memory, VirtualFiles, FileTokens) {
    let mut types = TypeRegistry::new();
    let file = types.reserve_record(RecordKind::Struct);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    types.define_record(file, [byte]).unwrap();
    let memory = Memory::new(Limits::default());
    let tokens = FileTokens::new(&types, file).unwrap();
    (types, file, memory, VirtualFiles::default(), tokens)
}
fn opened(files: &mut VirtualFiles) -> VirtualFileHandle {
    files
        .opened(
            HostPath::new(FileRootId::allocate(), "file").unwrap(),
            FileOpenMode::Read,
            b"bytes".to_vec(),
        )
        .unwrap()
}
#[test]
fn only_minted_tokens_survive_provenance_preserving_casts() {
    let (types, file, mut memory, mut files, mut tokens) = fixture();
    let handle = opened(&mut files);
    let token = tokens.mint(&types, &mut memory, handle).unwrap();
    assert_eq!(tokens.lookup(&types, &memory, &token).unwrap(), handle);
    let heap = memory.allocate(&types, file, None).unwrap();
    assert!(tokens.lookup(&types, &memory, &heap).is_err());
    let void = memory
        .cast_pointer(&types, &token, types.void(), CastMode::Unchecked)
        .unwrap();
    assert!(tokens.lookup(&types, &memory, &void).is_err());
    let restored = memory
        .cast_pointer(&types, &void, file, CastMode::Unchecked)
        .unwrap();
    assert_eq!(tokens.lookup(&types, &memory, &restored).unwrap(), handle);
    assert_eq!(memory.release(&token), Err(Error::ReadOnlyStorage));
    assert_eq!(memory.load(&types, &token), Err(Error::Uninitialized));
    let proof = tokens.remove(&types, &memory, &restored).unwrap();
    assert_eq!(proof.pointer(), &token);
    assert_eq!(proof.handle(), handle);
    assert!(tokens.lookup(&types, &memory, &token).is_err());
}
#[test]
fn other_memory_stale_and_shifted_pointers_do_not_select_a_file() {
    let (types, file, mut memory, mut files, mut tokens) = fixture();
    let handle = opened(&mut files);
    let token = tokens.mint(&types, &mut memory, handle).unwrap();
    let mut other = Memory::new(Limits::default());
    let wrong = other.allocate(&types, file, None).unwrap();
    assert_eq!(
        tokens.lookup(&types, &memory, &wrong),
        Err(Error::ForeignPointer)
    );
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let byte = memory
        .cast_pointer(&types, &token, u8, CastMode::Unchecked)
        .unwrap();
    let shifted = memory.offset(&types, &byte, 1).unwrap();
    let shifted = memory
        .cast_pointer(&types, &shifted, file, CastMode::Unchecked)
        .unwrap();
    assert!(tokens.lookup(&types, &memory, &shifted).is_err());
    let heap = memory
        .allocate(
            &types,
            u8,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 1))),
        )
        .unwrap();
    let guessed = memory
        .cast_pointer(&types, &heap, file, CastMode::Unchecked)
        .unwrap();
    assert!(tokens.lookup(&types, &memory, &guessed).is_err());
    memory.release(&heap).unwrap();
    assert_eq!(
        tokens.lookup(&types, &memory, &guessed),
        Err(Error::DanglingPointer)
    );
    tokens.reset();
    assert!(tokens.lookup(&types, &memory, &token).is_err());
}

#[test]
fn general_snapshot_permits_live_tokens_and_retains_empty_bucket_capacity() {
    let (types, _, mut memory, mut files, mut tokens) = fixture();
    let handle = opened(&mut files);
    let pointer = tokens.mint(&types, &mut memory, handle).unwrap();
    let expected = tokens.tokens.capacity() + 1 + 1 + pointer.metadata_cells();
    let mut charged = 0;
    assert_eq!(
        tokens
            .snapshot_bounds(&mut |work| {
                charged += work;
                Ok(())
            })
            .unwrap(),
        expected
    );
    assert_eq!(charged, (tokens.tokens.capacity() + 1) as u64);
    assert_eq!(tokens.lookup(&types, &memory, &pointer).unwrap(), handle);
    assert_eq!(memory.release(&pointer), Err(Error::ReadOnlyStorage));
    assert!(tokens.closed_table_capacity().is_err());
    assert_eq!(
        tokens.snapshot_bounds(&mut |_| Err(Error::Limit(crate::LimitKind::Fuel))),
        Err(Error::Limit(crate::LimitKind::Fuel))
    );
    assert_eq!(tokens.live_tokens(), 1);
    tokens.remove(&types, &memory, &pointer).unwrap();
    assert_eq!(
        tokens.snapshot_bounds(&mut |_| Ok(())).unwrap(),
        tokens.tokens.capacity() + 1
    );
}
