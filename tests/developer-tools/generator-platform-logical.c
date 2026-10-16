/* Independently authored identity controls; SDK-built actual native images. */
#ifndef IMAGE_VALUE
#define IMAGE_VALUE 37
#endif
int original_resolved(void) { return IMAGE_VALUE; }
#ifdef STATIC_VIEW
int original_static_only(void) { return 91; }
#endif
