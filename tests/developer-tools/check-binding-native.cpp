// Original declarations: native SDK manglings are the test input, never Jai source.
struct Original_Check_Record { int value; };
long long original_check_scalars(int a, unsigned short b, long c, double d, const unsigned char *e) {
  return a + b + c + static_cast<long long>(d) + *e;
}
int original_check_zero() { return 7; }
void original_check_reference(int &value) { ++value; }
void original_check_callback(int (*callback)(double)) { (void)callback(3); }
void original_check_array(int (&values)[4]) { ++values[0]; }
Original_Check_Record original_check_record(Original_Check_Record value) { return value; }
struct Original_Check_Method {
  int method(int a, double b);
};
int Original_Check_Method::method(int a, double b) { return a + static_cast<int>(b); }
