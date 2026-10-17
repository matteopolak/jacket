// Reviewed APK iteration explicitly rejects POINTER; compile rejection required.
A :: #import "Android/Toolchain";

main :: () {
    apk: A.Android_Apk;
    for *apk {}
}
