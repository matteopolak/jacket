#import "Basic";

scoped :: () {
    local: int;

    push_context {
        context.logger_data = *local;
    }
}

fresh :: () #c_call {
    local: int;

    push_context {
        context.logger_data = *local;
    }
}

escaping :: () {
    local: int;
    context.logger_data = *local;
}

mixed :: () -> *int, *int {
    local: int;
    return *local, null;
} @TempAllocates(1)

second :: () -> *int, *int {
    local: int;
    return null, *local;
} @TempAllocates(1)

heap_and_temp :: () -> *int, *int #foreign lib "heap_and_temp"; @Allocates(0) @TempAllocates(1)

lib :: #library "c";

bad_second :: () -> *int {
    heap, temporary := heap_and_temp();
    return temporary;
}

safe_first :: () -> *int {
    heap, temporary := heap_and_temp();
    return heap;
}

nested :: (flag: bool) {
    local: int;

    push_context {
        push_context {
            context.logger_data = *local;
        }

        context.logger_data = *local;

        if flag {
            push_context {
                context.logger_data = *local;
            }
        }

        context.logger_data = *local;
    }
}

restored_caller :: () {
    local: int;

    push_context {
        context.logger_data = *local;
    }

    context.logger_data = *local;
}

Node :: struct {
    values: [2] int;
}

heap_member :: () -> *int {
    node := New(Node);
    return *node.values[0];
}

local_literal :: () {
    a: int;
    b: int;
    pointers := (*int).[*a, *b];
    pointers[0].* = 1;
    pointers[1].* = 2;
}

bare_results :: () -> string, string {
    return tprint("%", 1), tprint("%", 2);
} @TempAllocates

restore_incoming :: () {
    local: int;
    saved := context.logger_data;
    defer context.logger_data = saved;
    context.logger_data = *local;
}
