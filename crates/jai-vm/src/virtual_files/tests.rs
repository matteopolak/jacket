use super::*;
use crate::{SourceOrigin, host_effects::FileRootId};
#[derive(Default)]
struct Effects {
    requests: Vec<HostRequest>,
    reject: bool,
}
impl HostEffects for Effects {
    fn begin(&mut self, _: SourceOrigin) -> Result<(), HostError> {
        Ok(())
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        self.requests.push(request);
        if self.reject {
            HostOutcome::Rejected(HostError::Denied("fixture"))
        } else {
            HostOutcome::Ready(HostResponse::WriteStaged)
        }
    }
    fn finish(&mut self, _: bool) -> Result<(), HostError> {
        Ok(())
    }
}
fn path() -> HostPath {
    HostPath::new(FileRootId::allocate(), "self-authored-data").unwrap()
}
#[test]
fn read_complete_items_and_eof_follow_stdio_rules() {
    let mut files = VirtualFiles::default();
    let file = files
        .opened(path(), FileOpenMode::Read, b"abcde".to_vec())
        .unwrap();
    let read = files.read(file, 2, 2).unwrap();
    assert_eq!(read.bytes, b"abcd");
    assert_eq!(read.complete_items, 2);
    assert!(!files.eof(file).unwrap());
    let read = files.read(file, 2, 1).unwrap();
    assert_eq!(read.bytes, b"e");
    assert_eq!(read.complete_items, 0);
    assert!(files.eof(file).unwrap());
    assert_eq!(files.tell(file).unwrap(), 5);
    files.seek(file, -1, SeekOrigin::End).unwrap();
    assert!(!files.eof(file).unwrap());
    assert_eq!(files.read(file, 1, 1).unwrap().bytes, b"e");
    assert!(!files.eof(file).unwrap());
    files.read(file, 1, 1).unwrap();
    assert!(files.eof(file).unwrap());
    assert_eq!(
        files.close(file, &mut Effects::default()).unwrap(),
        FileCloseOutcome::Closed
    );
    assert_eq!(files.tell(file), Err(FileError::UnknownHandle));
    assert_eq!(files.payload_bytes(), 0);
}
#[test]
fn truncated_writes_seek_holes_and_close_only_stage_bytes() {
    let mut files = VirtualFiles::default();
    let path = path();
    let file = files
        .opened(path.clone(), FileOpenMode::TruncateUpdate, vec![])
        .unwrap();
    assert_eq!(files.write(file, 2, 2, b"abcd").unwrap(), 2);
    files.seek(file, 6, SeekOrigin::Start).unwrap();
    files.write(file, 1, 1, b"x").unwrap();
    files.seek(file, 0, SeekOrigin::Start).unwrap();
    assert_eq!(files.read(file, 1, 7).unwrap().bytes, b"abcd\0\0x");
    let mut effects = Effects::default();
    assert_eq!(
        files.close(file, &mut effects).unwrap(),
        FileCloseOutcome::Closed
    );
    assert_eq!(
        effects.requests,
        vec![HostRequest::WriteEntireFile {
            path,
            bytes: b"abcd\0\0x".to_vec()
        }]
    );
    files.require_closed().unwrap();
}
#[test]
fn append_overrides_seek_for_each_write() {
    let mut files = VirtualFiles::default();
    let file = files
        .opened(path(), FileOpenMode::AppendUpdate, b"abc".to_vec())
        .unwrap();
    files.seek(file, 0, SeekOrigin::Start).unwrap();
    files.write(file, 1, 1, b"d").unwrap();
    assert_eq!(files.tell(file).unwrap(), 4);
    files.seek(file, 0, SeekOrigin::Start).unwrap();
    assert_eq!(files.read(file, 1, 4).unwrap().bytes, b"abcd");
}
#[test]
fn failures_do_not_mutate_buffers_and_stale_handles_never_revive() {
    let mut files = VirtualFiles::new(FileLimits {
        handles: 1,
        payload_bytes: 4,
        transfer_bytes: 4,
    });
    let file = files
        .opened(path(), FileOpenMode::TruncateUpdate, vec![])
        .unwrap();
    assert!(files.opened(path(), FileOpenMode::Read, vec![]).is_err());
    assert!(files.write(file, usize::MAX, 2, b"").is_err());
    files.write(file, 1, 4, b"data").unwrap();
    assert!(files.write(file, 1, 1, b"x").is_err());
    assert_eq!(files.tell(file).unwrap(), 4);
    assert!(files.seek(file, -5, SeekOrigin::Current).is_err());
    assert_eq!(files.tell(file).unwrap(), 4);
    assert_eq!(files.require_closed(), Err(FileError::LiveHandles));
    let mut effects = Effects {
        reject: true,
        ..Effects::default()
    };
    assert!(matches!(
        files.close(file, &mut effects).unwrap(),
        FileCloseOutcome::Rejected(_)
    ));
    assert_eq!(files.live_handles(), 1);
    files.reset();
    let next = files
        .opened(path(), FileOpenMode::Read, b"new".to_vec())
        .unwrap();
    assert_ne!(file, next);
    assert_eq!(files.read(file, 1, 1), Err(FileError::UnknownHandle));
    assert_eq!(files.write(next, 1, 1, b"x"), Err(FileError::ReadOnly));
}
#[test]
fn exact_mode_and_seek_decoding_and_zero_reads() {
    assert_eq!(FileOpenMode::parse(b"rb").unwrap(), FileOpenMode::Read);
    assert_eq!(
        FileOpenMode::parse(b"wb+").unwrap(),
        FileOpenMode::TruncateUpdate
    );
    assert_eq!(
        FileOpenMode::parse(b"a+").unwrap(),
        FileOpenMode::AppendUpdate
    );
    assert_eq!(
        FileOpenMode::parse(b"r-anything"),
        Err(FileError::UnsupportedMode)
    );
    assert_eq!(SeekOrigin::from_c(3), Err(FileError::InvalidSeek));
    let mut files = VirtualFiles::default();
    let file = files.opened(path(), FileOpenMode::Read, vec![]).unwrap();
    files.read(file, 0, usize::MAX).unwrap();
    assert!(!files.eof(file).unwrap());
    assert_eq!(files.tell(file).unwrap(), 0);
}

#[test]
fn general_snapshot_counts_live_reserved_backing_and_preserves_cursor() {
    let mut files = VirtualFiles::default();
    let mut bytes = Vec::with_capacity(8192);
    bytes.extend_from_slice(b"abc");
    let handle = files.opened(path(), FileOpenMode::Read, bytes).unwrap();
    files.seek(handle, 1, SeekOrigin::Start).unwrap();
    let state = &files.files[&handle];
    assert!(state.bytes.capacity() > state.bytes.len());
    let expected = files.files.capacity()
        + 1
        + 6
        + state.path.retained_path_capacity()
        + state.bytes.capacity();
    let mut charged = 0;
    let cells = files
        .snapshot_bounds(&mut |work| {
            charged += work;
            Ok(())
        })
        .unwrap();
    assert_eq!(cells, expected);
    assert_eq!(charged, (files.files.capacity() + 1) as u64);
    assert_eq!(files.tell(handle).unwrap(), 1);
    assert!(!files.eof(handle).unwrap());
    assert_eq!(files.payload_bytes(), 3);
    assert!(files.require_closed().is_err());
    assert_eq!(
        files.snapshot_bounds(&mut |_| Err(crate::Error::Limit(crate::LimitKind::Fuel))),
        Err(crate::Error::Limit(crate::LimitKind::Fuel))
    );
    assert_eq!(files.tell(handle).unwrap(), 1);
}
#[test]
fn general_snapshot_empty_tables_charge_retained_capacity_not_configured_maxima() {
    let mut files = VirtualFiles::new(FileLimits {
        handles: usize::MAX,
        payload_bytes: usize::MAX,
        transfer_bytes: usize::MAX,
    });
    assert_eq!(files.snapshot_bounds(&mut |_| Ok(())).unwrap(), 1);
    files.files.reserve(128);
    let expected = files.files.capacity() + 1;
    let mut charged = 0;
    assert_eq!(
        files
            .snapshot_bounds(&mut |work| {
                charged += work;
                Ok(())
            })
            .unwrap(),
        expected
    );
    assert_eq!(charged, expected as u64);
    assert_eq!(files.live_handles(), 0);
}
