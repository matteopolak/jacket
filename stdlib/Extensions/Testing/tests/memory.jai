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

temporary_storage_starts_empty :: () {
    expect_equal(context.temporary_storage.total_bytes_occupied, 0);
    used := tprint("%", 1);
    expect(used == "1");
} @test

temporary_storage_starts_empty_again :: () {
    expect_equal(context.temporary_storage.total_bytes_occupied, 0);
} @test
