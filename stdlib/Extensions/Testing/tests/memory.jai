// The tracking allocator: leaks and bad frees fail a test; temporary storage is fresh.
#import "Basic";
#import "Extensions/Testing";

freed_memory_passes :: () {
    memory := alloc(64);
    memory = realloc(memory, 128, 64);
    free(memory);
    text := copy_string("kept briefly");

    free(text);
} @test

a_leak_fails :: () {
    alloc(32);
} @test @should_fail(leaked)

a_leak_may_be_allowed :: () {
    alloc(32);
} @test @allow_leaks

a_double_free_fails :: () {
    memory := alloc(16);
    free(memory);
    free(memory); // jailint: allow(double_free) the bad free under test
} @test @should_fail(twice)

other_allocators_are_not_tracked :: () {
    _ := alloc(16,, temp);
    array: [..] int;
    array.allocator = temp;
    array_add(*array, 1);
} @test

heaps_are_not_offered :: () {
    // The tracker owns the allocator data, so it cannot hand out a heap: asking for one would
    // make a later call read a heap as a tracker.
    caps, _ := get_capabilities(context.allocator);

    expect((caps & .CREATE_HEAP) == 0);
    expect((caps & .FREE) != 0);
} @test

caps_without_a_name_slot :: () {
    // A capability query may pass no slot for the allocator's name.
    caps := cast(Allocator_Caps) cast(u64) context.allocator.proc(.CAPS, 0, 0, null, context.allocator.data);

    expect((caps & .FREE) != 0);
} @test

temporary_storage_starts_empty :: () {
    expect_equal(context.temporary_storage.total_bytes_occupied, 0);
    used := tprint("%", 1);
    expect(used == "1");
} @test

temporary_storage_starts_empty_again :: () {
    expect_equal(context.temporary_storage.total_bytes_occupied, 0);
} @test
