/* Corpus configuration for LVGL: 16-bit colour, built-in memory allocator, no logging. */
#ifndef LV_CONF_H
#define LV_CONF_H
#define LV_COLOR_DEPTH 16
#define LV_MEM_CUSTOM 0
#define LV_MEM_SIZE (32 * 1024)
#define LV_USE_LOG 0
#define LV_USE_OS LV_OS_NONE
#define LV_FONT_MONTSERRAT_14 1
#endif
