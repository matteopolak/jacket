#import "Basic";

Job :: struct {
    payload: *int;
}

Work :: #type (*Job);

register :: (job: *Job, work: Work) {} @RegistersBorrower(0,1,0)
start :: (job: *Job) {} @StartsBorrower(0)
finish :: (job: *Job) {} @EndsBorrower(0)
make :: () -> *int {
    return null;
} @Allocates
release :: (p: *int) {} @FreesArgument

copy_payload :: (job: *Job) {
    destination: int;
    memcpy(*destination, job.payload, size_of(int));
}

copy_nothing :: (job: *Job) {
    destination: int;
    memcpy(*destination, job.payload, 0);
}

bad :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, copy_payload);
    start(*job);
    release(p);
    finish(*job);
}

good :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, copy_nothing);
    start(*job);
    release(p);
    finish(*job);
}

write_payload :: (job: *Job) {
    memset(job.payload, 0, size_of(int));
}

write_nothing :: (job: *Job) {
    memset(job.payload, 0, 0);
}

bad_writer :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, write_payload);
    start(*job);
    release(p);
    finish(*job);
}

good_writer :: () {
    p := make();
    job: Job;
    job.payload = p;
    register(*job, write_nothing);
    start(*job);
    release(p);
    finish(*job);
}
