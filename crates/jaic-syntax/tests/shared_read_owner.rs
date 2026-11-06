//! Logical allocator deallocation, not process RSS or allocator cache residency.
use jaic_syntax::filesystem::ReadBuffer;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const LARGE: usize = 16 * 1024 * 1024;
static LIVE_LARGE: AtomicUsize = AtomicUsize::new(0);

struct CountLarge;

#[global_allocator]
static ALLOCATOR: CountLarge = CountLarge;

#[allow(unsafe_code)]
// SAFETY: allocations retain System's pointer/layout ownership; accounting neither allocates
// nor panics. Default zeroed/reallocation methods use these same allocation/deallocation hooks.
unsafe impl GlobalAlloc for CountLarge {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller supplies the valid allocation layout required by GlobalAlloc.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && layout.size() >= LARGE {
            LIVE_LARGE.fetch_add(layout.size(), Ordering::SeqCst);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if layout.size() >= LARGE {
            LIVE_LARGE.fetch_sub(layout.size(), Ordering::SeqCst);
        }
        // SAFETY: the caller returns the System pointer with its original allocation layout.
        unsafe { System.dealloc(pointer, layout) };
    }
}

#[test]
fn weak_read_owner_releases_payload_while_weak_slice_does_not() {
    let baseline = LIVE_LARGE.load(Ordering::SeqCst);
    let slice: Arc<[u8]> = Arc::from(vec![0; LARGE + 1]);
    let weak_slice = Arc::downgrade(&slice);
    let retained = LIVE_LARGE.load(Ordering::SeqCst);
    assert!(retained >= baseline + LARGE);
    drop(slice);
    assert!(weak_slice.upgrade().is_none());
    assert_eq!(
        LIVE_LARGE.load(Ordering::SeqCst),
        retained,
        "control: expired weak slice still retains the large backing allocation"
    );
    drop(weak_slice);
    assert_eq!(LIVE_LARGE.load(Ordering::SeqCst), baseline);

    let slice: Arc<[u8]> = Arc::from(vec![0; LARGE + 1]);
    let owner = Arc::new(ReadBuffer::new(slice));
    let proof_bytes = owner.bytes().clone();
    let proof_owner = owner.clone();
    let weak_owner = Arc::downgrade(&owner);
    drop(owner);
    drop(proof_bytes);
    assert!(LIVE_LARGE.load(Ordering::SeqCst) >= baseline + LARGE);
    drop(proof_owner);
    assert!(weak_owner.upgrade().is_none());
    assert_eq!(
        LIVE_LARGE.load(Ordering::SeqCst),
        baseline,
        "the payload is deallocated while the small weak owner still exists"
    );
    drop(weak_owner);
    assert_eq!(LIVE_LARGE.load(Ordering::SeqCst), baseline);
}
