// Executes only public record accessors in the host interpreter; no Windows syscall.
S :: #import "Socket";

#assert OS == .WINDOWS;
#assert #run verify();

verify :: () -> bool {
    scope: S.SCOPE_ID;
    scope.Value = 0xa1234567;
    if S.get_anon_struct_Zone(scope) != 0x1234567 || S.get_anon_struct_Level(scope) != 10 return false;

    S.set_anon_struct_Zone(*scope, 0xffffffff);
    if scope.Value != 0xafffffff || scope.anon_struct.__bitfield != scope.Value return false;

    S.set_anon_struct_Level(*scope, 0x12345678);

    return scope.Value == 0x8fffffff && scope.anon_struct.__bitfield == scope.Value;
}

main :: () {}
