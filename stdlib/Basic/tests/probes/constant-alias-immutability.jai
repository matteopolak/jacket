// Refreshed reviewed :: contract: initialization may reject or capture a constant,
// but assignment through the resulting declaration must never become a live store.
backing: u32 = 11;
Alias :: backing;

main :: () {
    Alias = 13;
}
