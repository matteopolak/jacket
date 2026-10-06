// Declarations of longdouble.c, for tests/stdlib/bindings-generator-long-double.jai.

struct LdBox { long double v; };
struct LdTagged { char tag; long double v; };
struct LdPair { long double a, b; };

long double ld_third(void);
long double ld_add(long double a, long double b);
long double ld_mix(int i, double d, long double x, long double y, int j);
double ld_to_double(long double x);
int ld_beyond_double(long double x);
struct LdBox box_scale(struct LdBox b, long double k);
struct LdTagged tagged_bump(struct LdTagged t);
struct LdPair pair_swap(struct LdPair p);
long double ld_spill(double d0, double d1, double d2, double d3, double d4, double d5, double d6,
                     double d7, double d8, long double x, int k, long double y);
long double ld_apply(long double (*f)(long double, long double), long double a, long double b);
struct LdBox box_apply(struct LdBox (*f)(struct LdBox, int), struct LdBox b);
