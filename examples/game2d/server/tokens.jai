// Account tokens. The stdlib has no cryptographic generator, so this seeds the fast `Random`
// generator with 16 bytes from the operating system and hands out 128 bits per token under a
// lock. That is enough for a dogfood server and is listed in FRICTION.md.

Token_Source :: struct {
    lock: Mutex;
    state: R.Random_State;
}

token_source_init :: (source: *Token_Source) -> bool {
    init(*source.lock);

    seed: [16] u8;
    file, opened := file_open("/dev/urandom", log_errors = false);

    if !opened return false;

    defer file_close(*file);

    ok, received := file_read(file, seed.data, seed.count);

    if !ok || received != seed.count return false;

    R.random_seed(*source.state, (cast(*u64) seed.data).*);
    R.random_seed(*source.state, (cast(*u64) (seed.data + 8)).* ^ R.random_get(*source.state));

    return true;
}

make_token :: (source: *Token_Source) -> string {
    lock(*source.lock);

    high := R.random_get(*source.state);
    low := R.random_get(*source.state);

    unlock(*source.lock);

    return tprint("%%", FormatInt.{ value = high, base = 16, minimum_digits = 16 }, FormatInt.{ value = low, base = 16, minimum_digits = 16 });
}

#scope_file

#import "Thread";
#import "File";
R :: #import "Random";
