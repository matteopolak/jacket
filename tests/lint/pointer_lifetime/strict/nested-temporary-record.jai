#import "Basic";
#import "Flat_Pool";

Failure :: struct {
    file: string;
    durable: string;
}

Result :: struct {
    failure: Failure;
    built: bool;
}

assemble :: () -> Result {
    failure: Failure;
    failure.file = tprint("%", "migration.sql");
    failure.durable = copy_string("durable");
    return .{ failure = failure };
}

saved: Result;

main :: () {
    saved = assemble();
    saved = durable_nested();
    saved = replaced_nested();
    local: Result;

    fill_output(*local);
    fill_output(*saved);
    scoped_allocator();
}

durable_nested :: () -> Result {
    source := assemble();
    failure := Failure.{ file = copy_string(source.failure.file), durable = copy_string("owned") };
    return .{ failure = failure };
}

replaced_nested :: () -> Result {
    result: Result;
    result.failure = .{ file = tprint("%", "discarded"), durable = "" };
    result = .{ failure = .{ file = copy_string("owned"), durable = "" } };
    return result;
}

fill_output :: (output: *Result) {
    output.* = assemble();
}

scoped_allocator :: () {
    pool: Flat_Pool;
    defer fini(*pool);
    push_allocator(flat_pool_allocator_proc, *pool);
}
