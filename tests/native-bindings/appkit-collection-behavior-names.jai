A :: #import "Objective_C/AppKit";

#assert cast(u64) A.NSWindowCollectionBehaviorFullScreenPrimary == 128;
#assert cast(u64) A.NSWindowCollectionBehaviorManaged == 4;
#assert type_of(A.NSWindowCollectionBehaviorDefault) == A.NSWindowCollectionBehavior;
#assert size_of(A.NSWindowCollectionBehavior) == 8;

probe :: #library,system "appkit_metal_contract_probe";
native_flag :: (index: u32) -> u64 #foreign probe "binding_collection_value";
libc :: #library,system "c";
abort :: () #foreign libc;

main :: () {
    values: [] A.NSWindowCollectionBehavior = .[
        .NSWindowCollectionBehaviorDefault, .NSWindowCollectionBehaviorCanJoinAllSpaces,
        .NSWindowCollectionBehaviorMoveToActiveSpace, .NSWindowCollectionBehaviorManaged,
        .NSWindowCollectionBehaviorTransient, .NSWindowCollectionBehaviorStationary,
        .NSWindowCollectionBehaviorParticipatesInCycle, .NSWindowCollectionBehaviorIgnoresCycle,
        .NSWindowCollectionBehaviorFullScreenPrimary, .NSWindowCollectionBehaviorFullScreenAuxiliary,
        .NSWindowCollectionBehaviorFullScreenNone, .NSWindowCollectionBehaviorFullScreenAllowsTiling,
        .NSWindowCollectionBehaviorFullScreenDisallowsTiling];

    for values {
        if cast(u64) it != native_flag(cast(u32) it_index) abort();
    }
}
