#include "developer-tools-point.h"

int64_t point_sum(Point *point) {
    point->samples[1] += 7;
    return point->x + (int64_t)point->y + point->samples[1];
}
