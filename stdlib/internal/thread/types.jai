Mutex_Order :: s32;
Thread_Index :: s64;
Thread_Proc :: #type (*Thread) -> s64;
POSIX_THREADS :: OS == .LINUX || OS == .MACOS || OS == .IOS || OS == .ANDROID;
SET_NAME_INSIDE_THREAD :: OS == .MACOS || OS == .IOS;

#if OS == .WINDOWS {
    Per_OS_Critical_Section_Type :: W.CRITICAL_SECTION;
    Per_OS_Condition_Variable_Type :: W.CONDITION_VARIABLE;
} else {
    Per_OS_Critical_Section_Type :: P.pthread_mutex_t;
    Per_OS_Condition_Variable_Type :: P.pthread_cond_t;
}

Wait_For_Result :: enum {
    SUCCESS;
    TIMEOUT;
    ERROR;
}

Mutex :: struct {
    csection: Per_OS_Critical_Section_Type;
    #if THREAD_DEBUG {
        initialized := false;
        name: string;
        order: Mutex_Order;
        redundant_lock_counter: s32;
        outer_name: string;
        outer_order: Mutex_Order;
        outer_csection: *Per_OS_Critical_Section_Type;
    }
}

Condition_Variable :: struct {
    condition_variable: Per_OS_Condition_Variable_Type;
    #if THREAD_DEBUG {
        initialized := false;
        name: string;
        current_mutex: *Mutex;
    }
}

#if THREAD_DEBUG {
    Mutex_Tracking :: struct {
        current_order: Mutex_Order = -1;
        current_order_set_by: Source_Code_Location;
        current_name := "";
        current_mutex_id: *Per_OS_Critical_Section_Type;
    }
}

#if OS == .WINDOWS {
    Semaphore :: struct {
        event: W.HANDLE;
    }
} else #if OS == .MACOS {
    Semaphore :: struct {
        owner: M.task_t;
        event: M.semaphore_t = 0;
    }
} else {
    Semaphore :: struct {
        semaphore: P.sem_t;
    }
}

#if OS == .WINDOWS {
    Thread_Os_Specific :: struct {
        windows_thread: W.HANDLE;
        windows_thread_id: s32;
    }
} else {
    Thread_Os_Specific :: struct {
        thread_handle: P.pthread_t;
        is_alive_semaphore: Semaphore;
        suspended_semaphore: Semaphore;
        is_done: bool;
        #if SET_THREAD_NAMES && SET_NAME_INSIDE_THREAD c_name: *u8;
    }
}

Thread :: struct {
    index: Thread_Index;
    proc: Thread_Proc;
    data: *void;
    workspace: Workspace;
    starting_context: #Context;
    starting_temporary_storage: Temporary_Storage;
    allocator_used_for_temporary_storage: Allocator;
    #if LOAD_THREAD_GROUP worker_info: *Thread_Group.Worker_Info;
    #if #exists(_STACK_TRACE) {
        #if _STACK_TRACE stack_trace_sentinel: Stack_Trace_Node;
    }
    using specific: Thread_Os_Specific;
}

Thread_Continue_Status :: enum {
    STOP :: 0;
    CONTINUE :: 1;
}

#if LOAD_THREAD_GROUP {
    Thread_Group_Proc :: #type (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status;

    Thread_Group :: struct {
        data: *void;
        proc: Thread_Group_Proc;
        name: string;
        logging := true;
        time_proc: () -> float64;
        allocator: Allocator;
        worker_info: [] Worker_Info;
        worker_info_data_to_free: *void;
        next_worker_index: s32;
        Worker_Info :: struct {
            using info: Unpadded_Worker_Info;
            INFO_SIZE :: size_of(type_of(info));
            PADDING_SIZE :: #run align_forward(INFO_SIZE, CACHE_LINE_SIZE) - INFO_SIZE;
            padding: [PADDING_SIZE] u8;
            work_steal_indices: [] s32;
        }
        Unpadded_Worker_Info :: struct {
            thread: Thread;
            available: Work_List;
            completed: Work_List;
            group: *Thread_Group;
            worker_index: s32;
        }
        Work_List :: struct {
            semaphore: Semaphore;
            mutex: Mutex;
            first: *Work_Entry;
            last: *Work_Entry;
            count: s32;
        }
        Work_Entry :: struct {
            next: *Work_Entry;
            work: *void;
            thread_index: Thread_Index;
            logging_name: string;
            issue_time: float64;
            work_list_index: s32;
        }
        initted := false;
        started := false;
        should_exit := false;
    }

    Work_List :: Thread_Group.Work_List;
    Work_Entry :: Thread_Group.Work_Entry;
}
