struct OriginalVirtualBase {
    int value = 3;
    virtual int first(int x) { return value + x; }
    virtual int hidden(int x) { return 40 + x; }
};
struct OriginalVirtualDerived : OriginalVirtualBase {
    using OriginalVirtualBase::hidden;
    int other = 7;
    int first(int x) override { return other + x; }
    virtual int hidden(int x) const { return 80 + x; }
    virtual int last(int x) { return 60 + x; }
};
struct OriginalVirtualDestructor { virtual ~OriginalVirtualDestructor() {} };
struct OriginalVirtualAligned {
    alignas(16) int value = 5;
    char tail = 7;
    virtual int first(int x) { return value + x; }
};
struct OriginalEmptyRecord { OriginalEmptyRecord() = default; };
struct OriginalOpaqueForward;
struct OriginalVirtualMultiple : OriginalVirtualBase, OriginalVirtualAligned {};
struct OriginalVirtualShared : virtual OriginalVirtualBase {};
struct OriginalVirtualVariadic { virtual int call(int x, ...) = 0; };
#pragma pack(push, 1)
struct OriginalPackedValue { char tag; int value; };
#pragma pack(pop)
struct OriginalVirtualPacked { virtual int call(OriginalPackedValue value) = 0; };
