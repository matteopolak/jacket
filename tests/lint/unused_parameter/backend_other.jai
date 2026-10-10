// Not loaded by the cases: a build-selected variant of `backend_step`, which `good.jai` also
// defines. Its name being defined here is what keeps the unused parameter there quiet.
backend_step :: (count: int, scale: int) -> int {
    return count * scale;
}
