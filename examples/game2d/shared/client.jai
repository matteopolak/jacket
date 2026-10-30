// A typed client for the API in protocol.jai, used by the game and by the load test. Calls are
// blocking when they run outside `Io.io_run` (the client starts a scheduler for the call) and
// ordinary fiber calls inside one.

Api :: struct {
    client: Client;
    tls: Tls.Tls_Config;
    account_id: s64;
    token: string;
}

// `roots_pem` is the certificate to trust for an https base URL (empty: the system's roots).
api_init :: (api: *Api, base_url: string, roots_pem := "") -> string {
    api.client.base_url = base_url;
    api.client.timeout = 5000;

    if roots_pem != "" {
        api.tls.roots_pem = roots_pem;
        api.tls.alpn = string.["http/1.1"];

        error := Tls.tls_config_init(*api.tls);

        if error.kind != .NONE return tprint("tls: %", Tls.error_text(error));

        api.client.tls = *api.tls;
    }

    return "";
}

// Reads the body of `response` as `T`, or turns a failure into text for the player.
api_read :: (response: *Client_Response, $T: Type) -> T, string {
    value: T;

    if response.status == 0 return value, response.reason;

    if response.status / 100 != 2 {
        text, _ := read_text(response);

        return value, tprint("HTTP %: %", response.status, text);
    }

    decoded, error := read_json(response, T);

    if error.kind != .NONE return value, error_text(error);

    return decoded, "";
}

api_health :: (api: *Api) -> Health, string {
    response, error := get(*api.client, ROUTE_HEALTH);

    if error.kind != .NONE return .{}, error_text(error);

    defer close(*response);

    return api_read(*response, Health);
}

api_create_account :: (api: *Api, name: string) -> Account, string {
    response, error := post(*api.client, ROUTE_ACCOUNTS, New_Account.{ name });

    if error.kind != .NONE return .{}, error_text(error);

    defer close(*response);

    account, failure := api_read(*response, Account);

    if !failure {
        api.account_id = account.id;
        api.token = copy_string(account.token);
    }

    return account, failure;
}

api_submit_match :: (api: *Api, match: New_Match) -> Match, string {
    signed := match;

    signed.checksum = match_checksum(match, api.token);

    auth := Header_Field.[.{ name = "Authorization", value = tprint("Bearer %", api.token) }];
    response, error := post(*api.client, ROUTE_MATCHES, signed, auth);

    if error.kind != .NONE return .{}, error_text(error);

    defer close(*response);

    return api_read(*response, Match);
}

api_leaderboard :: (api: *Api, limit: s64) -> [] Leaderboard_Entry, string {
    response, error := get(*api.client, tprint("%?limit=%", ROUTE_LEADERBOARD, limit));

    if error.kind != .NONE return .[], error_text(error);

    defer close(*response);

    return api_read(*response, [] Leaderboard_Entry);
}

#scope_module

#import "Basic";
#import "Extensions/Http";
Tls :: #import "Extensions/Tls";
