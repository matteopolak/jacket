// Exact instants, civil values, explicit zones and calendar-aware spans.

#load "types.jai";
#load "numeric.jai";
#load "civil.jai";
#load "instants.jai";
#load "spans.jai";
#load "zone_load.jai";
#load "zoned.jai";
#load "ordering.jai";
#load "text.jai";
#load "pattern.jai";
#load "interop.jai";

#scope_module

Basic :: #import "Basic";
#import "String";
Json :: #import "Extensions/Json";

using,except(Time_Zone) Basic;
NS :: 1000000000;
DAY_NS :: 86400000000000;
MIN_I64 :: -9223372036854775808;
MAX_I64 :: 9223372036854775807;
