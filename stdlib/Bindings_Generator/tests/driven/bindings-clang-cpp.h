class OriginalBase {
public:
    virtual int measure() const = 0;
    virtual ~OriginalBase() = default;
    int base_data;
};

class OriginalDerived final : public OriginalBase {
public:
    OriginalDerived() = default;
    OriginalDerived(const OriginalDerived &) = default;
    OriginalDerived(OriginalDerived &&) = default;
    explicit OriginalDerived(int value) : data(value) {}
    OriginalDerived &operator=(const OriginalDerived &) = default;
    OriginalDerived &operator=(OriginalDerived &&) = default;
    int measure() const override { return data; }
    static int utility(int x) { return x + 1; }
    int disabled() = delete;
    int data;
};

inline namespace OriginalInline {
extern thread_local int original_tls;
}
extern "C" int original_c_entry(int argument);
