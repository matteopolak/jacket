struct DispatchSmall { int a; };
struct DispatchBig { long long a, b, c; };
struct DispatchFloatPair { float a, b; };
struct DispatchEight { int a, b; };
struct DispatchSixteen { long long a, b; };
struct DispatchDoublePair { double a, b; };
struct DispatchBase {
    virtual int choose(int value) { return value + 10; }
    virtual int choose(float value) { return static_cast<int>(value) + 20; }
    virtual ~DispatchBase() {}
    virtual int tail(int value) { return value + 30; }
};
struct DispatchDerived : DispatchBase {
    int choose(int value) override { return value + 40; }
    virtual int extra(int value) { return value + 50; }
    virtual int extra(float value) { return static_cast<int>(value) + 60; }
    virtual DispatchSmall small(int value) { return {value + 70}; }
    virtual DispatchBig big(int value) { return {value + 80, value + 90, value + 100}; }
    virtual DispatchFloatPair pair(float value) { return {value + 1.5f, value + 2.5f}; }
    virtual DispatchEight eight(int value) { return {value + 110, value + 120}; }
    virtual DispatchSixteen sixteen(int value) { return {value + 130, value + 140}; }
    virtual DispatchDoublePair doubles(int value) { return {value + 5.25, value + 6.25}; }
    virtual int take(DispatchSmall a, DispatchBig b) { return a.a + static_cast<int>(b.a + b.b + b.c); }
};
