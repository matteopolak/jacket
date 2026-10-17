#include "bindings-input.h"

Counter sample_apply(struct Sample *sample, enum Mode mode) {
  sample->lanes[2] += 4;
  return sample->x + (Counter)sample->y + sample->lanes[2] + mode;
}

int callback_apply(Callback callback, int value) { return callback(value) + 2; }
