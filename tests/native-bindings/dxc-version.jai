// Reviewed Windows COM version helper owns its sprint result and frees the
// native custom-version string/interfaces. Real DXC required for execution.
#import "Basic";
D :: #import "dxc_compiler";

main :: () {
    version := D.dxc_get_version_string();
    assert(version.count >= 7 && version[0] == #char "v");
    assert(version[version.count - 1] == #char "]");
    free(version);
}
