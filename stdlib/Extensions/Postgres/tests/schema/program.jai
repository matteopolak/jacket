// Uses the structs `generate` wrote to db/schema.jai. Built by run.sh.
#import "Basic";
Pg :: #import "Extensions/Postgres";

Settings :: struct {
    theme: string;
}

#load "db/schema.jai";

main :: () {
    user: User;
    user.role = .admin;
    user.settings = Pg.some(Pg.json(Settings.{ theme = "dark" }));
    user.payload.value.text = "{}";

    team: Team;
    post: Post;
    event: Audit_Event;
    active: Active_User;

    assert(type_info(User).members.count == 8);
    assert(type_of(user.settings) == Pg.Nullable(Pg.Json(Settings)));
    assert(type_of(user.payload) == Pg.Json(Pg.Json_Raw));
    assert(user.settings.value.value.theme == "dark");
    assert(type_of(active.id) == Pg.Nullable(s64) && type_of(post.roles) == [] Member_Role && event.id == 0 && team.id == 0);

    assert(type_info(Team).notes.count == 2 && type_info(Team).notes[0] == "pg_table(public.teams)");
    assert(type_info(User).members[0].notes[0] == "pg_primary_key");

    print("ok\n");
}
