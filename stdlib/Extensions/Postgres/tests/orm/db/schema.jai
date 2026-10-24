// A hand-written stand-in for a generated db/schema.jai over tests/typed/schema.sql.
#scope_file

Pg :: #import "Extensions/Postgres";
#import "Extensions/Json";

#scope_export

Mood :: enum {
    sad;
    ok;
    happy;
}

Settings :: struct {
    theme: string;
    font_size := 14; @json(default)
    beta: Json_Maybe(bool);
}

User :: struct @pg_table(public.users) @pg_has_many(posts:Post:author_id:id) {
    id: s64; @pg_primary_key @pg_default
    email: string;
    display_name: Pg.Nullable(string);
    created_at: Pg.Timestamp; @pg_default
    settings: Pg.Nullable(Pg.Json(Settings));
    mood: Mood; @pg_default
    score: s32; @pg_default
    tags: [] string; @pg_default
}

Post :: struct @pg_table(public.posts) @pg_belongs_to(author:author_id:User:id) {
    id: s64; @pg_primary_key @pg_default
    author_id: s64;
    title: string;
    body: Pg.Nullable(string);
    created_at: Pg.Timestamp; @pg_default
}
