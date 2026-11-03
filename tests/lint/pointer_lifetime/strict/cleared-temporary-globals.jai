#import "Basic";

saved_text: string;

safe_cleared :: () {
    saved_text = tprint("%", "short lived");
    saved_text = "";
}

safe_deferred_clear :: () {
    defer saved_text = "";
    saved_text = tprint("%", "short lived");
}

safe_cleared_before_reset :: () {
    saved_text = tprint("%", "short lived");
    saved_text = "";
    reset_temporary_storage();
}

bad_retained :: () {
    saved_text = tprint("%", "retained");
}

bad_conditionally_cleared :: (clear: bool) {
    saved_text = tprint("%", "conditionally retained");
    if clear saved_text = "";
}

bad_reset_before_clear :: () {
    saved_text = tprint("%", "already invalidated");
    reset_temporary_storage();
    saved_text = "";
}

main :: () {}
