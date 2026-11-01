// A name that several namespaced imports export. The order the fixes are suggested in must not
// depend on how the compiler numbers the spellings.
Zed :: #import "Basic"; // jailint: allow(unused_import)
Alpha :: #import "Math"; // jailint: allow(unused_import)
Yak :: #import "Basic"; // jailint: allow(unused_import)
Mu :: #import "Math"; // jailint: allow(unused_import)
Xen :: #import "Basic"; // jailint: allow(unused_import)
Beta :: #import "Math"; // jailint: allow(unused_import)

main :: () {
    x := min(1, 2);
}
