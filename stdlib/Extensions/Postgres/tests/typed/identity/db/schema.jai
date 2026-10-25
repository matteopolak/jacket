// A hand-written stand-in for a generated db/schema.jai: the typed queries in ../main.jai find
// their table structs here by `@pg_table`.
#scope_file

Pg :: #import "Extensions/Postgres";

// Not visible to main.jai, so queries on posts keep their anonymous rows. Nothing names it: it is
// here to be passed over.
// jailint: allow(unused_private_declaration)
Hidden_Post :: struct @pg_table(public.posts) {
    id: s64;
    author_id: s64;
    title: string;
    body: Pg.Nullable(string);
    created_at: Pg.Timestamp;
}

#scope_export

Mood :: enum {
    sad;
    ok;
    happy;
}

User :: struct @pg_table(public.users) {
    id: s64; @pg_primary_key
    email: string;
    display_name: Pg.Nullable(string);
    created_at: Pg.Timestamp;
    settings: Pg.Nullable(Pg.Json(Pg.Json_Raw));
    mood: Mood;
    score: s32;
    tags: [] string;
}
