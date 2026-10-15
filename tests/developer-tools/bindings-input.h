typedef long long Counter;
typedef long NativeLong;
enum Mode { MODE_ADD = 3, MODE_SUBTRACT = 7 };
/** Original SDK-style record documentation. */
struct Sample {
  int x;
  double y;
  unsigned short lanes[3];
  struct Sample *next;
};
typedef struct Sample Sample;
union Payload {
  long long number;
  double real;
};
typedef int (*Callback)(int value);
Counter sample_apply(struct Sample *sample, enum Mode mode);
int callback_apply(Callback callback, int value);
#ifndef DEVELOPER_TOOLS_BINDINGS_INPUT_H
#define DEVELOPER_TOOLS_BINDINGS_INPUT_H
#endif
