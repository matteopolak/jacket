// A toy service for ../call.jai: upper-cases each request. Requests starting with "fail:" fail,
// "gone:" says the service is unavailable, "slow:" answers after 300 ms, and "crash" ends the
// process mid-request.

#import "Basic";
#import "String";
Service :: #import "Extensions/Service";

main :: () {
    Service.serve(answer);
}

answer :: (request: string) -> Service.Status, string {
    if begins_with(request, "fail:") return .FAILED, advance(request, 5);
    if begins_with(request, "gone:") return .UNAVAILABLE, advance(request, 5);
    if request == "crash" exit(3);

    if begins_with(request, "slow:") {
        sleep_milliseconds(300);
        return .OK, to_upper_copy(advance(request, 5),, temp);
    }

    return .OK, to_upper_copy(request,, temp);
}
