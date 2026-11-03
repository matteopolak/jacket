Job :: struct {
    payload: *int;
}

Work :: #type (*Job);

make :: () -> *int {
    return cast(*int) Basic.alloc(size_of(int));
}

release :: (p: *int) {} @FreesArgument
consume :: (p: *int) -> int {
    return p.*;
}

register :: (job: *Job, work: Work) {} @RegistersBorrower(0,1,0)
start :: (job: *Job) {} @StartsBorrower(0)
finish :: (job: *Job) {} @EndsBorrower(0)
done :: (job: *Job) -> bool {
    return false;
} @CompletesBorrower(0)
read_payload :: (job: *Job) {
    consume(job.payload);
}

no_payload :: (job: *Job) {}

bad_free_running :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, read_payload);
    start(*job);
    release(p);
    finish(*job);
}

bad_free_before_start :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, read_payload);
    release(p);
    start(*job);
    finish(*job);
}

bad_receiver_scope :: () {
    job: Job;
    register(*job, no_payload);
    start(*job);
}

good_finish :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, read_payload);
    start(*job);
    finish(*job);
    release(p);
}

good_completion :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, read_payload);
    start(*job);
    if done(*job) release(p);

    finish(*job);
}

good_unused_payload :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, no_payload);
    start(*job);
    release(p);
    finish(*job);
}

unknown_callback :: (callback: Work) {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, callback);
    start(*job);
    release(p);
    finish(*job);
}

conditional_read :: (job: *Job) {
    if job.payload != null consume(job.payload);
}

good_conditional_read_is_not_a_must_borrow :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, conditional_read);
    start(*job);
    release(p);
    finish(*job);
}

lib :: #library "c";
CWork :: #type (job: *Job) #c_call;

register_foreign :: (job: *Job, work: CWork) {} @RegistersBorrower(0,1,0)
foreign_callback :: (job: *Job) #foreign lib "lifetime_unavailable_worker"; @NoLifetimeEffects

foreign_read_contract_is_not_a_no_read_contract :: () {
    p := make();
    job: Job;
    job.payload = p;
    register_foreign(*job, foreign_callback);
    start(*job);
    release(p);
    finish(*job);
}

Basic :: #import "Basic";
