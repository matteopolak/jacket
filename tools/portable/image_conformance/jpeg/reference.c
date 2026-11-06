/* Independent pixel oracle through the public libjpeg API; no decoder implementation copied. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <setjmp.h>
#include <jpeglib.h>
#define STRINGIFY_INNER(x) #x
#define STRINGIFY(x) STRINGIFY_INNER(x)
struct error_state { struct jpeg_error_mgr pub; jmp_buf jump; };
static void fatal(j_common_ptr c) {
  (*c->err->output_message)(c);
  longjmp(((struct error_state *)c->err)->jump, 1);
}
int main(int argc, char **argv) {
  struct jpeg_decompress_struct c = {0};
  struct error_state e;
  FILE *input, *output = NULL;
  uint64_t limit;
  if (argc == 2 && !strcmp(argv[1], "--version")) {
#ifdef LIBJPEG_TURBO_VERSION
    printf("libjpeg-turbo %s API %d\n", STRINGIFY(LIBJPEG_TURBO_VERSION), JPEG_LIB_VERSION);
#else
    printf("libjpeg API %d\n", JPEG_LIB_VERSION);
#endif
    Dl_info library;
    if (!dladdr((void *)jpeg_std_error, &library) || !library.dli_fname) return 5;
    printf("LIBRARY %s\n", library.dli_fname);
    return 0;
  }
  if (argc != 4 || !(limit = strtoull(argv[3], NULL, 10))) return 4;
  input = fopen(argv[1], "rb");
  if (!input) return 4;
  c.err = jpeg_std_error(&e.pub); e.pub.error_exit = fatal;
  if (setjmp(e.jump)) {
    jpeg_destroy_decompress(&c); fclose(input);
    return 2;
  }
  jpeg_create_decompress(&c); jpeg_stdio_src(&c, input); jpeg_read_header(&c, TRUE);
  if ((uint64_t)c.image_width * c.image_height > limit) {
    printf("REFERENCE_CAP %u %u\n", c.image_width, c.image_height);
    jpeg_destroy_decompress(&c); fclose(input); return 3;
  }
  c.dct_method = JDCT_ISLOW; c.do_fancy_upsampling = TRUE;
  c.out_color_space = c.num_components == 4 ? JCS_CMYK : c.num_components == 1 ? JCS_GRAYSCALE : JCS_RGB;
  jpeg_start_decompress(&c);
  output = fopen(argv[2], "wb"); if (!output) return 4;
  printf("REF %u %u %d %d %d\n", c.output_width, c.output_height,
         c.output_components, c.saw_Adobe_marker, c.jpeg_color_space);
  JSAMPARRAY row = (*c.mem->alloc_sarray)((j_common_ptr)&c, JPOOL_IMAGE,
                      c.output_width * c.output_components, 1);
  while (c.output_scanline < c.output_height) {
    jpeg_read_scanlines(&c, row, 1);
    if (fwrite(row[0], c.output_width * c.output_components, 1, output) != 1) return 4;
  }
  jpeg_finish_decompress(&c);
  printf("WARN %ld\n", e.pub.num_warnings);
  fclose(output); fclose(input); jpeg_destroy_decompress(&c); return 0;
}
