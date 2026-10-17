// Reviewed linux_build requires a Windows host, even when targeting Windows.
L :: #import "linux_build";

main :: () {
    p := L.get_plugin();
    if p L.shutdown(p);
}
