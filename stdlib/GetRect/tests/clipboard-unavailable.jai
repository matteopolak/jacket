// test: skip_on(.MACOS, reason = "macOS has a clipboard provider; the test is the contract for hosts without")
// This private named-pasteboard provider is macOS-only; no substitute clipboard.
#import "Basic";
Transport :: #import "internal/numerics/platform_clipboard";

#assert OS != .MACOS;

main :: () {
    ok, message := Transport.write_text("caller text");

    assert(!ok);
    assert(message == "Native clipboard transport is unavailable for this target; no local substitute was used");
    text: string;
    ok, text, message = Transport.read_text();
    assert(!ok && text.count == 0);
    assert(message == "Native clipboard transport is unavailable for this target; no local substitute was used");
    print("named pasteboard provider explicitly unavailable\n");
}
