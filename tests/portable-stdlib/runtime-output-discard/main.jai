// Runtime-owned private buffering: discard removes pending bytes without I/O.
// This is host kernel verification, not claimed Android SDK/device execution.
using #import "Basic";
Output :: #import "internal/runtime/android-output";

main :: () {
    write_string("desktop immediate\n");
    Output.append("discarded stdout");
    Output.append("discarded stderr", true);
    Output.flush(false);
    Output.discard_pending_output();
    Output.flush(true);
    Output.discard_pending_output();
    Output.append("retained");
    Output.flush(false);
    Output.append(" line\n");
    Output.append("partial");
    Output.flush(true);
    Output.append(" done\n");
    Output.append("stderr line\n", true);
    bytes: [9000] u8;
    for *byte: bytes byte.* = 65;

    Output.append(.{ bytes.count, bytes.data });
    Output.discard_pending_output();
    Output.flush(true);
    write_string("\nend\n");
}
