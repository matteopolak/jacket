extern int gp_external(int);
static int gp_private(int x) { return x + 3; }
int gp_value = 17;
int gp_add(int x) { return gp_private(x) + gp_value; }
int gp_unresolved(int x) { return gp_external(x); }
