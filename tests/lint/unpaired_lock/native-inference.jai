lib :: #library,system "kernel32";
raw_enter :: (p: *void) #foreign lib "EnterCriticalSection";
raw_unknown :: (p: *void) #foreign lib "unknown_lock_effect";
raw_leave :: (p: *void) #foreign lib "LeaveCriticalSection";

// This annotation declares that returning held is intentional API behavior.
intentional_wrapper :: (p: *void) { raw_enter(p); } @Locks
inferred_release :: (p: *void) { raw_leave(p); }

ordinary_bug :: (p: *void) {
    raw_enter(p); // ordinary buggy function, not an intentional wrapper
}

nested_bug :: (p: *void) {
    intentional_wrapper(p);
    intentional_wrapper(p);
    inferred_release(p); // one recursive level is still held
}

nested_balanced :: (p: *void) {
    intentional_wrapper(p);
    intentional_wrapper(p);
    inferred_release(p);
    inferred_release(p);
}

balanced_helper :: (p: *void) {
    intentional_wrapper(p);
    inferred_release(p);
}

uses_balanced_helper :: (p: *void) { balanced_helper(p); }

deferred :: (p: *void) {
    intentional_wrapper(p);
    defer inferred_release(p);
}
nested_defer_bug :: (p: *void) {
    intentional_wrapper(p);
    defer inferred_release(p);
    intentional_wrapper(p); // one deferred release cannot balance two acquisitions
}

double_defer_balanced :: (p: *void) {
    intentional_wrapper(p);
    intentional_wrapper(p);
    defer { inferred_release(p); inferred_release(p); }
}

unknown_after_acquire :: (p: *void) {
    intentional_wrapper(p);
    raw_unknown(p); // might release or transfer the lock
}
Native_Lock :: struct { storage: [64] u8; }
raw_typed_enter :: (p: *Native_Lock) #foreign lib "EnterCriticalSection";
raw_typed_leave :: (p: *Native_Lock) #foreign lib "LeaveCriticalSection";
Two_Locks :: struct { first: Native_Lock; second: Native_Lock; }
first_acquire :: (p: *Two_Locks) { raw_typed_enter(*p.first); } @Locks
second_release :: (p: *Two_Locks) { raw_typed_leave(*p.second); }
wrong_internal_receiver :: (p: *Two_Locks) {
    first_acquire(p);
    second_release(p); // releases a different embedded lock
}

Aliased_Locks :: struct { first: *void; second: *void; }
first_pointer_acquire :: (p: *Aliased_Locks) { raw_enter(p.first); } @Locks
second_pointer_release :: (p: *Aliased_Locks) { raw_leave(p.second); }
unknown_pointer_alias :: (p: *Aliased_Locks) {
    first_pointer_acquire(p);
    second_pointer_release(p); // these pointer fields may name the same native lock
}
lock :: (p: *void) #foreign lib "EnterCriticalSection";
inferred_spelling_is_not_fix_intent :: (p: *void) {
    lock(p);
}
named_acquire :: (p: *void, other: *void) { raw_enter(p); } @Locks
reordered_named_binding_bug :: (a: *void, b: *void) {
    named_acquire(other = a, p = b);
    raw_leave(a);
}

indexed_wrapper :: (other: *void, p: *void) { raw_enter(p); } @Locks(1)
indexed_release :: (other: *void, p: *void) { raw_leave(p); } @Unlocks(1)
second_formal_binding_bug :: (a: *void, b: *void) {
    indexed_wrapper(a, b);
    raw_leave(a);
}
second_formal_balanced :: (a: *void, b: *void) {
    indexed_wrapper(other = a, p = b);
    indexed_release(p = b, other = a);
}
// Both inline native locks are initialized and unlocked on entry.
// The second one is held before it becomes the replacement receiver.
retargeted_receiver_bug :: (p: *Two_Locks) {
    holder: Aliased_Locks;
    holder.first = cast(*void) *p.first;
    raw_enter(cast(*void) *p.second);
    raw_enter(holder.first); // this first object is never released
    holder.first = cast(*void) *p.second;
    raw_leave(holder.first);
}

captured_receiver_balanced :: (p: *Two_Locks) {
    holder: Aliased_Locks;
    holder.first = cast(*void) *p.first;
    held := holder.first;
    raw_enter(held);
    holder.first = cast(*void) *p.second;
    raw_leave(held);
}
// Repeated event sites need recency; this slice leaves loop protocols unknown.
loop_protocol_unknown :: (p: *void, run: bool) {
    while run { raw_enter(p); }
}
release_registered_before_acquisition :: (p: *void) {
    defer raw_leave(p);
    raw_enter(p);
}
conditional_release_bug :: (p: *void, release: bool) {
    raw_enter(p);
    if release raw_leave(p);
}
replace_pointer :: (holder: *Aliased_Locks, next: *void) { holder.first = next; }
helper_retarget_unknown :: (p: *Two_Locks) {
    holder: Aliased_Locks;
    holder.first = cast(*void) *p.first;
    raw_enter(holder.first);
    replace_pointer(*holder, cast(*void) *p.second);
    raw_leave(holder.first); // not a proof that the earlier object was released
}
discharged_then_reacquired_bug :: (p: *void) {
    raw_enter(p); // this obligation is discharged
    raw_leave(p);
    raw_enter(p); // only this later acquisition leaks
}
main :: () {}
