#include <assert.h>
#include <dlfcn.h>
int main(int argc, char **argv) {
    assert(argc == 2);
    for (int i = 0; i < 2; ++i) {
        void *image = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
        assert(image);
        int (*value)(void) = (int (*)(void))dlsym(image, "runtime_ci_value");
        assert(value && value() == 73);
        assert(dlclose(image) == 0);
    }
    return 0;
}
