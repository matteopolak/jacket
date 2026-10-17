#include "developer-tools-public-bridge.cpp"
static_assert(__is_same(Binding_NativeBridge_choose_Arg0, long));
static_assert(__is_same(Binding_NativeBridge_choose_1_Arg0, long long));
static_assert(__is_same(Binding_NativeBridge_Box_add_Arg0, const NativeBridge::Box*));
static_assert(__is_same(Binding_NativeBridge_sum_Arg0, double));
static_assert(__is_same(Binding_NativeBridge_deep_Arg0, decltype(NativeBridge::deep(nullptr))));
int main() {
    if (NativeBridge_deep(nullptr) != nullptr) return 14;
    if (independently_generated_bindings_bridge::global_cpp() != 42) return 13;
    if (NativeBridge_sum(5.75) != 12 || NativeBridge_sum(5, 9) != 14) return 1;
    if (NativeBridge_nontrailing_default(5, 9) != 14) return 9;
    if (NativeBridge_choose(5) != 105 || NativeBridge_choose_1(5) != 205) return 2;
    NativeBridge::Pair pair;
    NativeBridge_make_pair(&pair, 18);
    if (pair.first != 18 || pair.second != 19) return 3;
    int* shared = NativeBridge_shared_value();
    if (*shared != 9) return 4;
    *shared = 23;
    if (*NativeBridge_shared_value() != 23) return 5;
    if (NativeBridge_bump(shared) != 27 || *shared != 27) return 10;
    const int values[] = {2, 3, 4};
    if (NativeBridge_sum_array(values) != 9) return 11;
    const int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}};
    if (NativeBridge_sum_matrix(matrix) != 21) return 12;
    alignas(NativeBridge::Box) unsigned char bytes[sizeof(NativeBridge::Box)];
    auto box = reinterpret_cast<NativeBridge::Box*>(bytes);
    NativeBridge_Box_Constructor(box, 30);
    if (NativeBridge_Box_add(box, 12) != 42 || NativeBridge_destruction_count() != 0) return 6;
    NativeBridge_Box_Destructor(box);
    if (NativeBridge_destruction_count() != 1) return 7;
    const char message[] = {'A', 0, 'B'};
    if (NativeBridge_message_message(3, message) != 131) return 8;
}
