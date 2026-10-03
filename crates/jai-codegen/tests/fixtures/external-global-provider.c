#include <stdint.h>

struct jai_test_external_state {
    int64_t total;
    int32_t status;
};

int64_t jai_test_external_counter = 20;
struct jai_test_external_state jai_test_external_state = {1, 3};

int64_t *jai_test_external_counter_address(void) {
    return &jai_test_external_counter;
}

struct jai_test_external_state *jai_test_external_state_address(void) {
    return &jai_test_external_state;
}

int32_t jai_test_external_provider_read(void) {
    return jai_test_external_counter == 21 &&
           jai_test_external_state.total == 21 &&
           jai_test_external_state.status == 7 ? 42 : 19;
}
