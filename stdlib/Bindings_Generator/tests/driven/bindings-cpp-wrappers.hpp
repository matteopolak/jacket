// An interface hierarchy whose methods are called only through vtable slots.
struct Point {
    int x;
    int y;
};

struct IRefCounted {
    virtual unsigned addRef() = 0;
    virtual unsigned release() = 0;
    virtual int queryInterface(int id, void** out) = 0;
};

struct IShape : public IRefCounted {
    virtual int area() const = 0;
    virtual int scale(int by) = 0;
    virtual int scale(const Point& by) = 0;
    virtual const Point& origin() const = 0;
    virtual Point& cursor() = 0;
    virtual Point offset(int delta) const = 0;
    virtual double ratio(double of, float by) const = 0;
    virtual int cast(int value) = 0;
};

struct IBox : public IShape {
    virtual int side() const = 0;
    virtual int split(IShape** first, IShape** second) = 0;
};

struct ICube : public IBox {
    virtual int volume() const = 0;
};
