W :: #import "Windows";
#import "Basic";

#assert size_of(W.SYSTEMTIME) == 16;
#assert align_of(W.SYSTEMTIME) == 2;
#assert offset_of(W.SYSTEMTIME, "wYear") == 0;
#assert offset_of(W.SYSTEMTIME, "wMonth") == 2;
#assert offset_of(W.SYSTEMTIME, "wDayOfWeek") == 4;
#assert offset_of(W.SYSTEMTIME, "wDay") == 6;
#assert offset_of(W.SYSTEMTIME, "wHour") == 8;
#assert offset_of(W.SYSTEMTIME, "wMinute") == 10;
#assert offset_of(W.SYSTEMTIME, "wSecond") == 12;
#assert offset_of(W.SYSTEMTIME, "wMilliseconds") == 14;
#assert size_of(W.TIME_ZONE_INFORMATION) == 172;
#assert align_of(W.TIME_ZONE_INFORMATION) == 4;
#assert offset_of(W.TIME_ZONE_INFORMATION, "Bias") == 0;
#assert offset_of(W.TIME_ZONE_INFORMATION, "StandardName") == 4;
#assert offset_of(W.TIME_ZONE_INFORMATION, "StandardDate") == 68;
#assert offset_of(W.TIME_ZONE_INFORMATION, "StandardBias") == 84;
#assert offset_of(W.TIME_ZONE_INFORMATION, "DaylightName") == 88;
#assert offset_of(W.TIME_ZONE_INFORMATION, "DaylightDate") == 152;
#assert offset_of(W.TIME_ZONE_INFORMATION, "DaylightBias") == 168;

main :: () {
    zone: W.TIME_ZONE_INFORMATION;

    #assert type_of(zone.Bias) == s32;
    #assert type_of(zone.StandardBias) == s32;
    #assert type_of(zone.DaylightBias) == s32;
    #assert type_of(zone.StandardName) == [32] u16;
    #assert type_of(zone.DaylightName) == [32] u16;
    #assert type_of(zone.StandardDate) == W.SYSTEMTIME;
    #assert type_of(zone.DaylightDate) == W.SYSTEMTIME;
    #assert type_of(zone.StandardDate.wYear) == W.WORD;
    #assert type_of(zone.StandardDate.wMonth) == W.WORD;
    #assert type_of(zone.StandardDate.wDayOfWeek) == W.WORD;
    #assert type_of(zone.StandardDate.wDay) == W.WORD;
    #assert type_of(zone.StandardDate.wHour) == W.WORD;
    #assert type_of(zone.StandardDate.wMinute) == W.WORD;
    #assert type_of(zone.StandardDate.wSecond) == W.WORD;
    #assert type_of(zone.StandardDate.wMilliseconds) == W.WORD;
    zone.Bias = -720;
    zone.StandardBias = -1;
    zone.DaylightBias = -60;
    zone.StandardName[31] = 0xffff;
    zone.DaylightName[31] = 0x8000;
    print("SYSTEMTIME.size %\nSYSTEMTIME.align %\n", size_of(W.SYSTEMTIME), align_of(W.SYSTEMTIME));
    print("TIME_ZONE_INFORMATION.size %\nTIME_ZONE_INFORMATION.align %\n", size_of(W.TIME_ZONE_INFORMATION), align_of(W.TIME_ZONE_INFORMATION));
    member("SYSTEMTIME.wYear", offset_of(W.SYSTEMTIME, "wYear"), size_of(u16));
    member("SYSTEMTIME.wMonth", offset_of(W.SYSTEMTIME, "wMonth"), size_of(u16));
    member("SYSTEMTIME.wDayOfWeek", offset_of(W.SYSTEMTIME, "wDayOfWeek"), size_of(u16));
    member("SYSTEMTIME.wDay", offset_of(W.SYSTEMTIME, "wDay"), size_of(u16));
    member("SYSTEMTIME.wHour", offset_of(W.SYSTEMTIME, "wHour"), size_of(u16));
    member("SYSTEMTIME.wMinute", offset_of(W.SYSTEMTIME, "wMinute"), size_of(u16));
    member("SYSTEMTIME.wSecond", offset_of(W.SYSTEMTIME, "wSecond"), size_of(u16));
    member("SYSTEMTIME.wMilliseconds", offset_of(W.SYSTEMTIME, "wMilliseconds"), size_of(u16));
    member("TIME_ZONE_INFORMATION.Bias", offset_of(W.TIME_ZONE_INFORMATION, "Bias"), size_of(type_of(zone.Bias)));
    member("TIME_ZONE_INFORMATION.StandardName", offset_of(W.TIME_ZONE_INFORMATION, "StandardName"), size_of(type_of(zone.StandardName)));
    member("TIME_ZONE_INFORMATION.StandardDate", offset_of(W.TIME_ZONE_INFORMATION, "StandardDate"), size_of(type_of(zone.StandardDate)));
    member("TIME_ZONE_INFORMATION.StandardBias", offset_of(W.TIME_ZONE_INFORMATION, "StandardBias"), size_of(type_of(zone.StandardBias)));
    member("TIME_ZONE_INFORMATION.DaylightName", offset_of(W.TIME_ZONE_INFORMATION, "DaylightName"), size_of(type_of(zone.DaylightName)));
    member("TIME_ZONE_INFORMATION.DaylightDate", offset_of(W.TIME_ZONE_INFORMATION, "DaylightDate"), size_of(type_of(zone.DaylightDate)));
    member("TIME_ZONE_INFORMATION.DaylightBias", offset_of(W.TIME_ZONE_INFORMATION, "DaylightBias"), size_of(type_of(zone.DaylightBias)));
    print("Bias.signed %\nStandardBias.signed %\nDaylightBias.signed %\n", cast(s32) (zone.Bias < 0), cast(s32) (zone.StandardBias < 0), cast(s32) (zone.DaylightBias < 0));
    print("StandardName.count %\nDaylightName.count %\n", zone.StandardName.count, zone.DaylightName.count);
}

member :: (name: string, offset: s64, bytes: s64) {
    print("%.offset %\n%.size %\n", name, offset, name, bytes);
}
