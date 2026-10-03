Profile_Tracker_Data_Record :: struct {
    self_time: History_Scalar;
    hierarchical_time: History_Scalar;
    entry_count: History_Scalar;
    max_recursion: s64;
}

Zone_Profiling_Data :: struct {
    t_self_start: s64;
    total_self_ticks: s64;
    total_hier_ticks: s64;
    total_entry_count: u32;
    parent: *Zone_Profiling_Data;
    zone: *Zone;
    recursion_depth: s32;
    profile_tracker_data_record: *Profile_Tracker_Data_Record;
}

count_recursion_depth :: (stack: *Zone_Profiling_Data, zone: *Zone) -> s32 {
    if !zone return 0;

    slow := stack;
    fast := stack;

    while fast && fast.parent {
        slow = slow.parent;
        fast = fast.parent.parent;
        if slow == fast {
            log_error("Iprof: cyclic profiling parent chain\n");
            return -1;
        }
    }

    result: s32;
    node := stack;

    while node {
        if node.zone == zone {
            if result == 0x7fffffff {
                log_error("Iprof: recursion depth is not representable\n");
                return -1;
            }

            result += 1;
        }

        node = node.parent;
    }

    return result;
}

#scope_file
#import "Basic";
