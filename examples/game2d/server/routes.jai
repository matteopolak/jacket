// The JSON API: accounts, matches and the leaderboard. Handlers declare what they need as
// parameters; the Http module reads the path, the query, the headers and the JSON body before the
// handler runs and answers 400/415/422 itself when they do not fit.

Game_Server :: struct {
    db: Database;
    tokens: Token_Source;
    started_ms: s64;
}

// `Authorization: Bearer <token>`: the token, or an empty string.
bearer_token :: (header: string) -> string {
    prefix :: "Bearer ";

    if header.count <= prefix.count || !begins_with(header, prefix) return "";

    return slice(header, prefix.count, header.count - prefix.count);
}

valid_name :: (name: string) -> bool {
    if name.count == 0 || name.count > MAX_NAME_LENGTH return false;

    for c: name {
        letter := (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z");
        digit := c >= #char "0" && c <= #char "9";

        if !letter && !digit && c != #char "_" && c != #char "-" return false;
    }

    return true;
}

health :: (server: *State(Game_Server)) -> Health, Http_Error {
    connection, error := borrow(*server.db);

    if error.kind != .NONE return .{}, error;

    defer give_back(*server.db, connection);

    accounts, count_error := scalar(connection, "select count(*) from accounts", 0);

    if count_error.kind != .NONE return .{}, count_error;

    matches: s64;

    matches, count_error = scalar(connection, "select count(*) from matches", 0);

    if count_error.kind != .NONE return .{}, count_error;

    return .{ true, API_VERSION, accounts, matches }, .{};
}

create_account :: (body: Json(New_Account), server: *State(Game_Server)) -> Response, Http_Error {
    name := body.temporary.name;

    if !valid_name(name) {
        return .{}, unprocessable("name must be 1 to % letters, digits, - or _", MAX_NAME_LENGTH);
    }

    connection, error := borrow(*server.db);

    if error.kind != .NONE return .{}, error;

    defer give_back(*server.db, connection);

    token := make_token(*server.tokens);
    result, pg_error := Pg.query_dynamic(connection, "insert into accounts (name, token) values ($1, $2) returning id", name, token);

    defer Pg.free_result(*result);

    if pg_error.code == .UNIQUE_VIOLATION return .{}, conflict("the name % is taken", name);
    if pg_error.kind != .NONE return .{}, database_failed(pg_error);

    id, decode_error := Pg.get(*result, 0, 0, s64);

    if decode_error.kind != .NONE return .{}, database_failed(decode_error);

    account := Account.{ id, copy_string(name,, temp), token };

    return created(account, tprint("%/%", ROUTE_ACCOUNTS, id)), .{};
}

get_account :: (id: Path("id", s64), server: *State(Game_Server)) -> Account_Public, Http_Error {
    connection, error := borrow(*server.db);

    if error.kind != .NONE return .{}, error;

    defer give_back(*server.db, connection);

    sql :: "select a.name, count(m.id), coalesce(max(m.score), 0) from accounts a left join matches m on m.account_id = a.id where a.id = $1 group by a.id";
    result, pg_error := Pg.query_dynamic(connection, sql, id.value);

    defer Pg.free_result(*result);

    if pg_error.kind != .NONE return .{}, database_failed(pg_error);
    if result.row_count == 0 return .{}, not_found("no account %", id.value);

    name, _ := Pg.get(*result, 0, 0, string);
    matches, _ := Pg.get(*result, 0, 1, s64);
    best, _ := Pg.get(*result, 0, 2, s64);

    return .{ id.value, copy_string(name,, temp), matches, best }, .{};
}

submit_match :: (body: Json(New_Match), authorization: Header_Optional("authorization", string), server: *State(Game_Server)) -> Response, Http_Error {
    submitted := body.temporary;
    token := bearer_token(authorization.value);

    if !token return .{}, unauthorized("send the account token as Authorization: Bearer <token>");

    ok, why := plausible(submitted);

    if !ok return .{}, unprocessable("this match cannot have happened: %", why);

    connection, error := borrow(*server.db);

    if error.kind != .NONE return .{}, error;

    defer give_back(*server.db, connection);

    account, pg_error := Pg.query_dynamic(connection, "select id, name from accounts where token = $1", token);

    defer Pg.free_result(*account);

    if pg_error.kind != .NONE return .{}, database_failed(pg_error);
    if account.row_count == 0 return .{}, unauthorized("unknown token");

    account_id, _ := Pg.get(*account, 0, 0, s64);
    player, _ := Pg.get(*account, 0, 1, string);

    if submitted.checksum != match_checksum(submitted, token) {
        return .{}, unprocessable("the checksum does not match the match");
    }

    sql :: "insert into matches (account_id, seed, score, kills, duration_ms) values ($1, $2, $3, $4, $5) returning id";
    inserted, insert_error := Pg.query_dynamic(connection, sql, account_id, submitted.seed, submitted.score, submitted.kills, submitted.duration_ms);

    defer Pg.free_result(*inserted);

    if insert_error.kind != .NONE return .{}, database_failed(insert_error);

    id, _ := Pg.get(*inserted, 0, 0, s64);
    better, count_error := scalar(connection, "select count(distinct account_id) from matches where score > $1", 0, submitted.score);

    if count_error.kind != .NONE return .{}, count_error;

    match := Match.{ id, account_id, copy_string(player,, temp), submitted.seed, submitted.score, submitted.kills, submitted.duration_ms, better + 1 };

    return created(match, tprint("%/%", ROUTE_MATCHES, id)), .{};
}

get_match :: (id: Path("id", s64), server: *State(Game_Server)) -> Match, Http_Error {
    connection, error := borrow(*server.db);

    if error.kind != .NONE return .{}, error;

    defer give_back(*server.db, connection);

    sql :: "select m.id, m.account_id, a.name, m.seed, m.score, m.kills, m.duration_ms from matches m join accounts a on a.id = m.account_id where m.id = $1";
    result, pg_error := Pg.query_dynamic(connection, sql, id.value);

    defer Pg.free_result(*result);

    if pg_error.kind != .NONE return .{}, database_failed(pg_error);
    if result.row_count == 0 return .{}, not_found("no match %", id.value);

    match: Match;

    match.id, _ = Pg.get(*result, 0, 0, s64);
    match.account_id, _ = Pg.get(*result, 0, 1, s64);

    player, _ := Pg.get(*result, 0, 2, string);

    match.player = copy_string(player,, temp);
    match.seed, _ = Pg.get(*result, 0, 3, s64);
    match.score, _ = Pg.get(*result, 0, 4, s64);
    match.kills, _ = Pg.get(*result, 0, 5, s64);
    match.duration_ms, _ = Pg.get(*result, 0, 6, s64);

    better, count_error := scalar(connection, "select count(distinct account_id) from matches where score > $1", 0, match.score);

    if count_error.kind != .NONE return .{}, count_error;

    match.rank = better + 1;

    return match, .{};
}

Leaderboard_Query :: struct {
    limit := DEFAULT_LEADERBOARD; @json(default)
}

leaderboard :: (query: Query(Leaderboard_Query), server: *State(Game_Server)) -> [] Leaderboard_Entry, Http_Error {
    limit := query.temporary.limit;

    if limit < 1 || limit > MAX_LEADERBOARD return .[], unprocessable("limit must be 1 to %", MAX_LEADERBOARD);

    connection, error := borrow(*server.db);

    if error.kind != .NONE return .[], error;

    defer give_back(*server.db, connection);

    sql :: "select a.name, max(m.score) as best, count(*) from matches m join accounts a on a.id = m.account_id group by a.id order by best desc, min(m.id) limit $1";
    result, pg_error := Pg.query_dynamic(connection, sql, limit);

    defer Pg.free_result(*result);

    if pg_error.kind != .NONE return .[], database_failed(pg_error);

    entries: [] Leaderboard_Entry;

    entries.count = result.row_count;
    entries.data = alloc(result.row_count * size_of(Leaderboard_Entry),, temp);

    for 0..result.row_count - 1 {
        name, _ := Pg.get(*result, it, 0, string);
        best, _ := Pg.get(*result, it, 1, s64);
        count, _ := Pg.get(*result, it, 2, s64);

        entries[it] = .{ it + 1, copy_string(name,, temp), best, count };
    }

    return entries, .{};
}

server_routes :: (app: *App, server: *Game_Server) {
    app.state = server;

    get(app, ROUTE_HEALTH, health);
    post(app, ROUTE_ACCOUNTS, create_account);
    get(app, ROUTE_ACCOUNT, get_account);
    post(app, ROUTE_MATCHES, submit_match);
    get(app, ROUTE_MATCH, get_match);
    get(app, ROUTE_LEADERBOARD, leaderboard);
}
