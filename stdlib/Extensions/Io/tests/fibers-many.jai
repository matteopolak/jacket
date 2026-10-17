// Ten thousand fibers alive at once, each switching several times, plus fibers that spawn fibers.
// Fibers share one thread and run in turn, so plain globals need no locks.
#import "Basic";
Io :: #import "Extensions/Io";

FIBERS :: 10000;

group: Io.Wait_Group;
started: int;
finished: int;
switches: int;
sum_of_indexes: int;

worker :: (index: int) {
    started += 1;

    for 1..5 {
        switches += 1;
        Io.yield();
    }

    sum_of_indexes += index;
    finished += 1;
    Io.done(*group);
}

// A binary tree of fibers: each one spawns two children until `depth` runs out.
tree_nodes: int;
tree_group: Io.Wait_Group;

tree :: (depth: int) {
    tree_nodes += 1;

    if depth > 0 {
        Io.add(*tree_group, 2);
        Io.spawn(tree, depth - 1);
        Io.spawn(tree, depth - 1);
    }

    Io.yield();
    Io.done(*tree_group);
}

// A fiber that is spawned from a fiber that ends before it runs.
chain_length: int;

chain :: (remaining: int) {
    chain_length += 1;
    if remaining > 0 Io.spawn(chain, remaining - 1);
}

root :: () {
    Io.add(*group, FIBERS);
    for 0..FIBERS - 1 Io.spawn(worker, it);

    // Nothing has run yet: spawning does not switch.
    assert(started == 0, "spawn must not run the new fiber at once, started = %", started);

    error := Io.wait(*group);

    assert(error == .NONE);
    assert(started == FIBERS, "started %", started);
    assert(finished == FIBERS, "finished %", finished);
    assert(switches == FIBERS * 5, "switches %", switches);
    assert(sum_of_indexes == FIBERS * (FIBERS - 1) / 2, "sum %", sum_of_indexes);

    // 2^11 - 1 fibers.
    Io.add(*tree_group, 1);
    Io.spawn(tree, 10);
    Io.wait(*tree_group);
    assert(tree_nodes == 2047, "tree nodes %", tree_nodes);

    Io.spawn(chain, 999);
    for 1..1100 Io.yield();

    assert(chain_length == 1000, "chain %", chain_length);

    // The pool reuses the stacks of fibers that ended: another round costs no new ones.
    finished = 0;
    Io.add(*group, 2000);
    for 0..1999 Io.spawn(worker, it);

    Io.wait(*group);
    assert(finished == 2000, "finished again %", finished);
}

main :: () {
    Io.io_run(root);
    print("ok\n");
}
