template<typename T, typename U> struct OriginalGeneric {
    T value;
    T *next;
    const U *other;
    U identity(T argument);
};
template<int N> struct OriginalNonType { int value; };
template<typename... T> struct OriginalPack { int value; };
OriginalGeneric<int, double> *original_instance;
const OriginalGeneric<int, double> *original_const_instance;
OriginalGeneric<OriginalGeneric<int, double> *, const int> *original_nested_instance;
OriginalGeneric<int, double> original_concrete;
struct OriginalTwoBases : OriginalGeneric<int, double>, OriginalGeneric<double, int> {};
OriginalGeneric<int, double> original_template_aggregate = { 1, nullptr, nullptr };
namespace OriginalTemplateOverloads {
void equivalent(OriginalGeneric<int, double> *);
void equivalent(OriginalGeneric<int, double> &);
void distinct(OriginalGeneric<int, double> *);
void distinct(OriginalGeneric<double, int> *);
}
template<typename V, typename W> struct OriginalOuter { OriginalGeneric<V, W> *inner; };
template<typename T> struct OriginalRecursive { OriginalRecursive<T> *next; };
