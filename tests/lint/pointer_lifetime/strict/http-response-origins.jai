#import "Basic";
Http :: #import "Extensions/Http";

json_saved: Http.Response;
problem_saved: Http.Response;

Envelope :: struct {
    payload: [] u8;
}

saved_payload: Envelope;

returned_payload :: () -> Envelope {
    text := tprint("%", "payload");
    result: Envelope;
    result.payload = .{ text.count, text.data };
    return result;
}

main :: () {
    saved_payload = returned_payload();
    json_saved = Http.json(42);
    problem_saved = Http.problem_response(null, Http.http_error(400, "bad request"));
}
