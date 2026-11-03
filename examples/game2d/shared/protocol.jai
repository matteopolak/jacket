API_VERSION :: 1;

// Routes, written once. The server registers them and the client builds its URLs from them.
ROUTE_HEALTH :: "/api/health";
ROUTE_ACCOUNTS :: "/api/accounts";
ROUTE_ACCOUNT :: "/api/accounts/{id}";
ROUTE_MATCHES :: "/api/matches";
ROUTE_MATCH :: "/api/matches/{id}";
ROUTE_LEADERBOARD :: "/api/leaderboard";

MAX_NAME_LENGTH :: 24;
MAX_LEADERBOARD :: 100;
DEFAULT_LEADERBOARD :: 10;

// A player's name is the only thing the server asks for when an account is made.
New_Account :: struct {
    name: string;
}

Account :: struct {
    id: s64;
    name: string;
    token: string;
}

// What anyone may see of an account: the token is only ever returned when it is created.
Account_Public :: struct {
    id: s64;
    name: string;
    matches: s64;
    best_score: s64;
}

// A finished match as the game reports it. `seed` is the level seed: the same seed gives the same
// waves. `checksum` signs the numbers with the account's token (`match_checksum`).
New_Match :: struct {
    seed: s64;
    score: s64;
    kills: s64;
    duration_ms: s64;
    checksum: string;
}

Match :: struct {
    id: s64;
    account_id: s64;
    player: string;
    seed: s64;
    score: s64;
    kills: s64;
    duration_ms: s64;
    rank: s64;
}

Leaderboard_Entry :: struct {
    rank: s64;
    player: string;
    best_score: s64;
    matches: s64;
}

Health :: struct {
    ok: bool;
    version: s64;
    accounts: s64;
    matches: s64;
}

// A score the game could not have produced in the time the match took is refused. The limits are
// generous: this guards against typos and curl, not against a determined cheater.
MAX_POINTS_PER_SECOND :: 400;
MAX_SCORE :: 10_000_000;

plausible :: (m: New_Match) -> bool, string {
    if m.score < 0 || m.kills < 0 || m.duration_ms < 0 return false, "negative numbers";
    if m.score > MAX_SCORE return false, "score too large";
    if m.duration_ms == 0 return m.score == 0, "a score without any time";
    if m.score * 1000 > m.duration_ms * MAX_POINTS_PER_SECOND return false, "score too high for the duration";
    if m.score > m.kills * MAX_POINTS_PER_KILL return false, "score without the kills for it";

    return true, "";
}

// The signature of a match: xxHash64 over its numbers and the token. It tells a mangled body from
// a good one, and a body made without the token from one made with it. (xxHash is not a MAC; a
// real service would use HMAC. This is a dogfood game.)
match_checksum :: (m: New_Match, token: string) -> string {
    state: XXH64_state_t;

    XXH64_reset(*state, cast(u64) API_VERSION);

    numbers := s64.[m.seed, m.score, m.kills, m.duration_ms];

    XXH64_update(*state, numbers.data, cast(u32) (numbers.count * size_of(s64)));
    XXH64_update(*state, token.data, cast(u32) token.count);

    return tprint("%", FormatInt.{ value = XXH64_digest(*state), base = 16, minimum_digits = 16 });
}

#scope_module

#import "Basic";
#import "xxHash";
