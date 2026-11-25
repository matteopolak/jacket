#import "Basic";
#import "Extensions/Testing";
#import "slugs";

short_title :: (b: *Bench) {
    title := "Hello, World!";

    for b {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench

long_title :: (b: *Bench) {
    title := "The Quick Brown Fox Jumps Over The Lazy Dog, Again And Again";
    set_bytes(b, title.count);

    for b {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench @samples(30)

many_titles :: (b: *Bench) {
    titles: [..] string;
    for 0..99 array_add(*titles, tprint("Title Number %", it));
    set_items(b, titles.count);

    for b {
        for titles {
            slug := slugify(it);
            black_box(slug.count);
            free(slug);
        }
    }
} @bench @min_time(500)
