#include "bindings-cpp-wrappers.hpp"
#include "bindings-cpp-wrappers-slang.hpp"

#if defined(_WIN32)
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif

namespace {

int live_objects = 0;

class CubeImpl final : public ICube {
public:
    CubeImpl(int side, int x, int y) : side_(side), origin_{x, y}, cursor_{0, 0} { live_objects += 1; }

    unsigned addRef() override { return ++references_; }
    unsigned release() override {
        unsigned remaining = --references_;
        if (remaining == 0) {
            live_objects -= 1;
            delete this;
        }
        return remaining;
    }
    int queryInterface(int id, void** out) override {
        if (id < 1 || id > 4) return -1;
        *out = static_cast<ICube*>(this);
        addRef();
        return 0;
    }
    int area() const override { return 6 * side_ * side_; }
    int scale(int by) override { side_ *= by; return side_; }
    int scale(const Point& by) override { side_ *= by.x * by.y; return side_; }
    const Point& origin() const override { return origin_; }
    Point& cursor() override { return cursor_; }
    Point offset(int delta) const override { return Point{origin_.x + delta, origin_.y + delta}; }
    double ratio(double of, float by) const override { return of * by + side_; }
    int cast(int value) override { return value + side_; }
    int side() const override { return side_; }
    int split(IShape** first, IShape** second) override {
        *first = new CubeImpl(side_ / 2, origin_.x, origin_.y);
        *second = new CubeImpl(side_ - side_ / 2, origin_.x, origin_.y);
        (*first)->addRef();
        (*second)->addRef();
        return 0;
    }
    int volume() const override { return side_ * side_ * side_; }

private:
    unsigned references_ = 0;
    int side_;
    Point origin_;
    Point cursor_;
};

class SessionImpl final : public slang::ISession {
public:
    SessionImpl(int index, SessionDesc desc) : index_(index), desc_(desc) { live_objects += 1; }
    SlangResult queryInterface(SlangUUID const& uuid, void** outObject) override {
        if (uuid.data1 != 7) return -1;
        *outObject = static_cast<slang::ISession*>(this);
        addRef();
        return 0;
    }
    unsigned addRef() override { return ++references_; }
    unsigned release() override {
        unsigned remaining = --references_;
        if (remaining == 0) {
            live_objects -= 1;
            delete this;
        }
        return remaining;
    }
    int getIndex() override { return index_; }
    SlangResult describe(SessionDesc* outDesc) override { *outDesc = desc_; return 0; }

private:
    unsigned references_ = 1;
    int index_;
    SessionDesc desc_;
};

class GlobalImpl final : public slang::IGlobalSession {
public:
    GlobalImpl() { live_objects += 1; }
    SlangResult queryInterface(SlangUUID const& uuid, void** outObject) override {
        if (uuid.data1 != 9) return -1;
        *outObject = static_cast<slang::IGlobalSession*>(this);
        addRef();
        return 0;
    }
    unsigned addRef() override { return ++references_; }
    unsigned release() override {
        unsigned remaining = --references_;
        if (remaining == 0) {
            live_objects -= 1;
            delete this;
        }
        return remaining;
    }
    SlangResult createSession(SessionDesc const& desc, slang::ISession** outSession) override {
        *outSession = new SessionImpl(++created_, desc);
        return 0;
    }
    const char* getBuildTagString() override { return "wrappers-1"; }

private:
    unsigned references_ = 1;
    int created_ = 0;
};

}

extern "C" EXPORT ICube* wrappers_create_cube(int side, int x, int y) {
    CubeImpl* cube = new CubeImpl(side, x, y);
    cube->addRef();
    return cube;
}

extern "C" EXPORT slang::IGlobalSession* wrappers_create_global_session() { return new GlobalImpl; }

extern "C" EXPORT int wrappers_live_objects() { return live_objects; }
