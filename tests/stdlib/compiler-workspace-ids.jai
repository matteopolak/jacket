// rules: ws.1 ws.2 ws.3 ws.4
#import "Compiler";

// The program being compiled is workspace 2 (workspace 1 is reserved), so the first
// workspace a metaprogram creates is 3.

current :: #run get_current_workspace();
first :: #run compiler_create_workspace("child");
second :: #run compiler_create_workspace();

main :: () -> s64 {
    if current != 2 return 1;
    if first != 3 return 2;
    if second != 4 return 3;
    return 0;
}
