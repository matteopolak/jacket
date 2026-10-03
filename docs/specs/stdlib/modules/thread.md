# Thread


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Mutex_Order :: s32`
- [DEBUG] `Mutex_Tracking :: struct`
- `Thread_Index :: s64`
- `Thread_Proc :: #type (*Thread) -> s64`
- `Thread :: struct`
- `thread_init_handler: #type (thread: *Thread)`
- `POSIX_THREADS :: (OS == .LINUX || OS == .MACOS || OS == .IOS || OS == .ANDROID)`
- [OS == .WINDOWS || OS == .PS5 || POSIX_THREADS || OS == .NN_SWITCH || OS == .NN_SWITCH2] `thread_init :: (thread: *Thread, proc: Thread_Proc, temporary_storage_size : s32 = 16384, starting_storage: *Temporary_Storage = null, name := "") -> bool`
- [OS == .WINDOWS || OS == .PS5 || POSIX_THREADS || OS == .NN_SWITCH || OS == .NN_SWITCH2] `thread_deinit :: (thread: *Thread)`
- [OS == .WINDOWS] `Per_OS_Critical_Section_Type :: CRITICAL_SECTION`
- [OS == .WINDOWS] `Per_OS_Condition_Variable_Type :: CONDITION_VARIABLE`
- [POSIX_THREADS] `Per_OS_Critical_Section_Type :: pthread_mutex_t`
- [POSIX_THREADS] `Per_OS_Condition_Variable_Type :: pthread_cond_t`
- `Mutex :: struct`
- `Wait_For_Result :: enum`
- `Condition_Variable :: struct`
- [POSIX_THREADS] `SET_NAME_INSIDE_THREAD :: OS == .MACOS || OS == .IOS`
- [POSIX_THREADS] `Thread_Os_Specific :: struct`
- [POSIX_THREADS] `thread_start :: (thread: *Thread)`
- [POSIX_THREADS] `thread_is_done :: (thread: *Thread, milliseconds: s32 = 0) -> bool`
- [POSIX_THREADS] `destroy :: (using m: *Mutex)`
- [POSIX_THREADS] `init :: (using m: *Mutex, desired_name := "", desired_order: Mutex_Order = -1)`
- [POSIX_THREADS] `init :: (using cv : *Condition_Variable, desired_name := "")`
- [POSIX_THREADS] `destroy :: (using cv : *Condition_Variable)`
- [POSIX_THREADS && OS == .MACOS] `Semaphore :: struct`
- [POSIX_THREADS && OS == .MACOS] `init :: (s: *Semaphore, initial_value: s32 = 0)`
- [POSIX_THREADS && OS == .MACOS] `destroy :: (s: *Semaphore)`
- [POSIX_THREADS && OS == .MACOS] `signal :: (s: *Semaphore)`
- [POSIX_THREADS && OS == .MACOS] `wait_for :: (s: *Semaphore, milliseconds: s32 = -1) -> Wait_For_Result #no_context`
- [POSIX_THREADS && else branch] `Semaphore :: struct`
- [POSIX_THREADS && else branch] `init :: (s: *Semaphore, initial_value: s32 = 0)`
- [POSIX_THREADS && else branch] `destroy :: (s: *Semaphore)`
- [POSIX_THREADS && else branch] `signal :: (s: *Semaphore)`
- [POSIX_THREADS && else branch] `wait_for :: (s: *Semaphore, milliseconds: s32 = -1) -> Wait_For_Result #no_context`
- [OS == .WINDOWS] `Thread_Os_Specific :: struct`
- [OS == .WINDOWS] `thread_start :: (thread: *Thread)`
- [OS == .WINDOWS] `thread_is_done :: (thread: *Thread, milliseconds: s32 = 0) -> bool`
- [OS == .WINDOWS] `destroy :: (using m: *Mutex)`
- [OS == .WINDOWS] `init :: (using m: *Mutex, desired_name := "", desired_order: Mutex_Order = -1)`
- [OS == .WINDOWS] `Semaphore :: struct`
- [OS == .WINDOWS] `init :: (s: *Semaphore, initial_value: s32 = 0)`
- [OS == .WINDOWS] `destroy :: (s: *Semaphore)`
- [OS == .WINDOWS] `signal :: (s: *Semaphore)`
- [OS == .WINDOWS] `wait_for :: (s: *Semaphore, milliseconds : s32 = -1) -> Wait_For_Result`
- [OS == .WINDOWS] `init :: (using cv : *Condition_Variable, desired_name := "")`
- [OS == .WINDOWS] `destroy :: (using cv : *Condition_Variable)`
- [OS == .WINDOWS || POSIX_THREADS] `csection_lock :: (csection: *Per_OS_Critical_Section_Type) #expand`
- [OS == .WINDOWS || POSIX_THREADS] `csection_unlock :: (csection: *Per_OS_Critical_Section_Type) #expand`
- [OS == .WINDOWS || POSIX_THREADS] `cvariable_wait :: (condition_variable : *Per_OS_Condition_Variable_Type, csection : *Per_OS_Critical_Section_Type, wait_time_ms : s32 = -1)`
- [OS == .WINDOWS || POSIX_THREADS] `cvariable_wake :: (condition_variable : *Per_OS_Condition_Variable_Type)`
- [OS == .WINDOWS || POSIX_THREADS] `cvariable_wake_all :: (condition_variable : *Per_OS_Condition_Variable_Type)`
- [DEBUG] `lock :: (using m: *Mutex, loc := #caller_location)`
- [DEBUG] `unlock :: (using m: *Mutex, loc := #caller_location)`
- [else branch] `lock :: (using m: *Mutex)`
- [else branch] `unlock :: (using m: *Mutex)`
- [DEBUG] `wait :: (c: *Condition_Variable, m: *Mutex, loc := #caller_location)`
- [else branch] `wait :: (c: *Condition_Variable, m: *Mutex)`
- `wake :: (c: *Condition_Variable)`
- `wake_all :: (c: *Condition_Variable)`
- `Thread_Group_Proc :: #type (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status`
- `Thread_Continue_Status :: enum`
- `Thread_Group :: struct`
- `Work_List :: Thread_Group.Work_List`
- `Work_Entry :: Thread_Group.Work_Entry`
- `init :: (using group: *Thread_Group, num_threads: s32, group_proc: Thread_Group_Proc, enable_work_stealing := false, temporary_storage_size : s32 = 16384, group_name := "")`
- `start :: (group: *Thread_Group)`
- `shutdown :: (using group: *Thread_Group, timeout_milliseconds: s32 = -1) -> bool`
- `add_work :: (group: *Thread_Group, work: *void, logging_name := "")`
- `get_completed_work :: (group: *Thread_Group) -> [] *void`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Condition_Variable`

```jai
Condition_Variable :: struct {
    condition_variable : Per_OS_Condition_Variable_Type;

    #if DEBUG {
        initialized := false;

        name:           string;
        current_mutex:  *Mutex;
    }
}
```
### `Mutex`

```jai
Mutex :: struct {
    csection: Per_OS_Critical_Section_Type;

    #if DEBUG {
        initialized := false;

        name:        string;
        order:       Mutex_Order;

        redundant_lock_counter: s32;

        outer_name:     string;
        outer_order:    Mutex_Order;
        outer_csection: *Per_OS_Critical_Section_Type;
    }
}
```
### `Mutex_Tracking`

```jai
Mutex_Tracking :: struct {
        current_order        : Mutex_Order = -1;
        current_order_set_by : Source_Code_Location;
        current_name         := "";
        current_mutex_id     : *Per_OS_Critical_Section_Type;
    }
```
### `Semaphore`

Variant 1 (common):

```jai
Semaphore :: struct {
            owner: task_t;
            event: semaphore_t = 0;
        }
```
Variant 2 (common):

```jai
Semaphore :: struct {
            semaphore: sem_t;
        }
```
Variant 3 (common):

```jai
Semaphore :: struct {
        event: HANDLE;
    }
```
### `Thread`

```jai
Thread :: struct {
    index : Thread_Index;
    proc  : Thread_Proc;
    data  : *void;
    workspace : Workspace;

    starting_context: #Context;
    starting_temporary_storage: Temporary_Storage;

    allocator_used_for_temporary_storage: Allocator;

    #if LOAD_THREAD_GROUP worker_info: *Thread_Group.Worker_Info;

    #if _STACK_TRACE  stack_trace_sentinel: Stack_Trace_Node;

    using specific : Thread_Os_Specific;
}
```
### `Thread_Continue_Status`

```jai
Thread_Continue_Status :: enum {
    STOP     :: 0;
    CONTINUE :: 1;
}
```
### `Thread_Group`

```jai
Thread_Group :: struct {

    data: *void;
    proc: Thread_Group_Proc;
    name: string;
    logging := true;

    time_proc: () -> float64;

    allocator:      Allocator;

    worker_info: [] Worker_Info;
    worker_info_data_to_free: *void;

    next_worker_index: s32;

    Worker_Info :: struct {
        using info: Unpadded_Worker_Info;

        INFO_SIZE    :: size_of(type_of(info));
        PADDING_SIZE :: #run Basic.align_forward(INFO_SIZE, CACHE_LINE_SIZE) - INFO_SIZE;

        padding: [PADDING_SIZE] u8;

        work_steal_indices: [] s32;
    }

    Unpadded_Worker_Info :: struct {
        thread:       Thread;
        available:    Work_List;
        completed:    Work_List;

        group:        *Thread_Group;
        worker_index: s32;
    }

    Work_List :: struct {
        semaphore: Semaphore;
        mutex:     Mutex;
        first:     *Work_Entry;
        last:      *Work_Entry;

        count:    s32;
    }

    Work_Entry :: struct {
        next:            *Work_Entry;
        work:            *void;
        thread_index:    Thread_Index;
        logging_name:    string;

        issue_time:      float64;
        work_list_index: s32;
    }

    initted     := false;
    started     := false;
    should_exit := false;
}
```
### `Thread_Os_Specific`

Variant 1 (common):

```jai
Thread_Os_Specific :: struct {
        thread_handle: pthread_t;
        is_alive_semaphore:  Semaphore;
        suspended_semaphore: Semaphore;
        is_done: bool;
        #if SET_THREAD_NAMES && SET_NAME_INSIDE_THREAD {
            c_name: *u8;
        }
    }
```
Variant 2 (common):

```jai
Thread_Os_Specific :: struct {
        windows_thread:    HANDLE;
        windows_thread_id: s32;
    }
```
### `Wait_For_Result`

```jai
Wait_For_Result :: enum {
    SUCCESS;
    TIMEOUT;
    ERROR;
}
```
## Behavioral contract

`thread_init` prepares a suspended thread with its own context, thread index, temporary storage, optional stack-trace chain, and user procedure; `thread_start` begins it. Keep the `Thread` object, starting allocator, and temporary storage alive until `thread_is_done` reports completion, then call `thread_deinit`. Mutexes, condition variables, and semaphores expose platform-backed synchronization; timeout `-1` means wait indefinitely, and semaphore waits return a `Wait_For_Result`. Mutex tracking/order checks are debug aids. OS-specific implementations cover Windows and selected POSIX/mobile/console targets; availability is conditional on target. `start` on a group that is not initialized, or already started, does nothing. `Thread_Group` routes opaque work pointers to worker callbacks and returns completed pointers to the caller; work storage remains caller-owned and must outlive execution. Explicit `shutdown` joins workers before shared state is torn down. Depends on OS threading primitives, allocators, context, and optional race detector.

### Operation-level behavior

| Operation group | Observable behavior |
| --- | --- |
| `thread_init`, `thread_start` | `thread_init` prepares the callback/context and creates a thread that remains gated until `thread_start`; it returns false if OS thread creation fails. If caller-supplied temporary storage is provided, its value is copied into the thread's storage record; later changes to the caller's record are not reflected. Otherwise storage of `temporary_storage_size` is allocated. |
| `thread_is_done` | `milliseconds = 0` polls; `-1` waits indefinitely; a positive value bounds the wait. Returns true only when completion is observed on POSIX; Windows reports true for any wait result other than timeout, so invalid-handle errors are not separately represented. Values below `-1` log an error but are not a defined timeout mode. |
| `thread_deinit` | Joins/reaps the worker on POSIX and releases the thread's temporary storage, then resets its index. Call only after the worker has completed; on Windows the handle is closed, so the caller should likewise establish completion first. |
| Semaphores | `init` requires a nonnegative initial count. `signal` adds one permit; `wait_for` consumes a permit on success and returns `TIMEOUT` or `ERROR` when applicable. Negative timeout means indefinite wait. |
| Mutexes and condition variables | These wrap native synchronization primitives. Initialize before use and destroy after all users finish. On POSIX the mutex is recursive. Condition-variable waits use the supplied native mutex/critical section; callers must follow the target OS's lock/wait discipline. |

Thread completion and resource ownership are separate: a callback returning does not make it safe to free the `Thread`, its allocator, or its temporary storage until completion has been observed and teardown/join has occurred.

## Module parameters

```jai
#module_parameters (LOAD_THREAD_GROUP := true)(CACHE_LINE_SIZE : s32 = 64, DO_RACE_DETECTOR := false, SET_THREAD_NAMES := true);
```
Platform coverage note: Nintendo, PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.
