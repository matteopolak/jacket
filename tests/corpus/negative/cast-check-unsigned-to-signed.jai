// An unsigned value above the signed maximum does not fit a signed type of the same size.
work :: () {
    u: u64 = 0xffff_ffff_ffff_fff4;
    c := cast(s64) u;
}
#run work();
