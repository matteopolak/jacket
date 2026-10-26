// test: check_only
// Has no `main`: it is only type-checked, so the test passes when it type-checks.
sum :: (values: [] s64) -> s64 {
    total: s64;
    for values total += it;

    return total;
}
