// A hand-written stand-in for a generated db/schema.jai over tests/typed/schema.sql.
#scope_file

Pg :: #import "Extensions/Postgres";

#scope_export

Mood :: enum {
    sad;
    ok;
    happy;
}

Settings :: struct {
    theme: string;
    font_size := 14; @json(default)
}

User :: struct @pg_table(public.users) {
    id: s64; @pg_primary_key @pg_default
    email: string;
    display_name: Pg.Nullable(string);
    created_at: Pg.Timestamp; @pg_default
    settings: Pg.Nullable(Pg.Json(Settings));
    mood: Mood; @pg_default
    score: s32; @pg_default
    tags: [] string; @pg_default
}

Post :: struct @pg_table(public.posts) {
    id: s64; @pg_primary_key @pg_default
    author_id: s64;
    title: string;
    body: Pg.Nullable(string);
    created_at: Pg.Timestamp; @pg_default
}
