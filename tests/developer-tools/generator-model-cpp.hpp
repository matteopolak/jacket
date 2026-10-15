namespace OriginalModelSpace {
struct Item {
    int value;
    Item();
    ~Item();
    int read(int) const;
    static int run(int);
    virtual int abstract_read(const int &) const = 0;
    int with_ref(const int &);
private:
    int secret;
public:
    int (*factory())(int);
    operator bool() const;
};
int duplicate(int);
int duplicate(int);
int equivalent(int *);
int equivalent(int &);
}
namespace OriginalModelSpace {
struct Later;
int before_definition(Later *);
struct Later { int member; };
namespace Nested { int first; }
}
namespace OriginalModelSpace { namespace Nested { int second; } }
int original_standalone(int named);
