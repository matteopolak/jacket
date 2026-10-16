#include <stdint.h>
#include <stdio.h>

int32_t binding_stream_control_cell = 19;

FILE **binding_stream_address(uint32_t index) {
  switch (index) {
  case 0:
    return &stdin;
  case 1:
    return &stdout;
  case 2:
    return &stderr;
  default:
    return NULL;
  }
}

FILE *binding_stream_value(uint32_t index) {
  FILE **cell = binding_stream_address(index);
  return cell ? *cell : NULL;
}

void binding_stream_assign(uint32_t index, FILE *value) {
  FILE **cell = binding_stream_address(index);
  if (cell)
    *cell = value;
}

int32_t *binding_stream_cell_address(void) {
  return &binding_stream_control_cell;
}
