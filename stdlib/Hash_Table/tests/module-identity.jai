#import "Basic";
H1 :: #import "Hash_Table";
H2 :: #import "Hash_Table"(COUNT_COLLISIONS = false);
P1 :: #import "Pool";
P2 :: #import "Pool"(USE_UNMAPPING_ALLOCATOR = false);
T1 :: #import "Thread";
T2 :: #import "Thread"(LOAD_THREAD_GROUP = true);

#assert H1.Table(string, s64) != H2.Table(string, s64);
#assert P1.Pool != P2.Pool;
#assert T1.Thread != T2.Thread;

main :: () {
    assert(type_info(P1.Pool).name == "Pool" && type_info(P2.Pool).name == "Pool");
    print("ok\n");
}
