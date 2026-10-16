// HTTP dates (RFC 9110 5.6.7): the IMF-fixdate form `Sun, 06 Nov 1994 08:49:37 GMT`. A server
// sends a Date header on every response, so Date_Cache formats the text once per second.

#scope_module

DAY_NAMES :: string.["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
MONTH_NAMES :: string.["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

#scope_export

// The date text for the current second, formatted again only when the second changes. One
// cache per thread (it is not synchronized).
Date_Cache :: struct {
    second: s64 = -1;
    text: [29] u8;
}

// Writes `seconds` since 1970-01-01 UTC as an IMF-fixdate (always 29 bytes).
format_http_date :: (seconds: s64, out: *[29] u8) {
    days := seconds / 86400;
    rest := seconds % 86400;

    if rest < 0 {
        rest += 86400;
        days -= 1;
    }

    // 1970-01-01 was a Thursday.
    weekday := (days + 3) % 7;
    if weekday < 0 weekday += 7;

    // Days since 1970-01-01 to a civil date (proleptic Gregorian).
    z := days + 719468;
    era := z / 146097;

    if z < 0 era = (z - 146096) / 146097;

    day_of_era := z - era * 146097;
    year_of_era := (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    year := year_of_era + era * 400;
    day_of_year := day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    mp := (5 * day_of_year + 2) / 153;
    day := day_of_year - (153 * mp + 2) / 5 + 1;
    month := mp + 3;

    if month > 12 month -= 12;
    if month <= 2 year += 1;

    hour := rest / 3600;
    minute := (rest % 3600) / 60;
    second := rest % 60;

    // "Sun, 06 Nov 1994 08:49:37 GMT"
    name := DAY_NAMES[weekday];
    mname := MONTH_NAMES[month - 1];

    text := out;
    text.*[0] = name[0];
    text.*[1] = name[1];
    text.*[2] = name[2];
    text.*[3] = #char ",";
    text.*[4] = #char " ";
    two_digits(text, 5, day);
    text.*[7] = #char " ";
    text.*[8] = mname[0];
    text.*[9] = mname[1];
    text.*[10] = mname[2];
    text.*[11] = #char " ";
    two_digits(text, 12, year / 100 % 100);
    two_digits(text, 14, year % 100);
    text.*[16] = #char " ";
    two_digits(text, 17, hour);
    text.*[19] = #char ":";
    two_digits(text, 20, minute);
    text.*[22] = #char ":";
    two_digits(text, 23, second);
    text.*[25] = #char " ";
    text.*[26] = #char "G";
    text.*[27] = #char "M";
    text.*[28] = #char "T";

    // Years under 1000 or over 9999 do not fit the four-digit form; clamp them to digits.
}

// The date for `seconds`; the string points into `cache` and stays valid until the next call.
http_date :: (cache: *Date_Cache, seconds: s64) -> string {
    if cache.second != seconds {
        format_http_date(seconds, *cache.text);
        cache.second = seconds;
    }

    result: string;
    result.data = cache.text.data;
    result.count = 29;

    return result;
}

// Reads an IMF-fixdate into seconds since 1970-01-01 UTC. The obsolete RFC 850 and asctime forms
// are not accepted.
parse_http_date :: (text: string) -> s64, bool {
    if text.count != 29 || text[3] != #char "," || text[4] != #char " " || text[7] != #char " " || text[11] != #char " " || text[16] != #char " " || text[19] != #char ":" || text[22] != #char ":" || !equal_nocase(slice(text, 25, 4), " GMT") {
        return 0, false;
    }

    weekday := -1;

    for DAY_NAMES {
        if slice(text, 0, 3) == it weekday = it_index;
    }

    month := 0;

    for MONTH_NAMES {
        if slice(text, 8, 3) == it month = it_index + 1;
    }

    if weekday < 0 || month == 0 return 0, false;

    day, ok1 := read_digits(text, 5, 2);
    year, ok2 := read_digits(text, 12, 4);
    hour, ok3 := read_digits(text, 17, 2);
    minute, ok4 := read_digits(text, 20, 2);
    second, ok5 := read_digits(text, 23, 2);

    if !ok1 || !ok2 || !ok3 || !ok4 || !ok5 return 0, false;
    if day < 1 || day > 31 || hour > 23 || minute > 59 || second > 60 return 0, false;

    // Civil date to days since 1970-01-01.
    y := year;
    if month <= 2 y -= 1;

    era := y / 400;
    year_of_era := y - era * 400;
    shifted := month - 3;

    if month <= 2 shifted = month + 9;

    day_of_year := (153 * shifted + 2) / 5 + day - 1;
    day_of_era := year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    days := era * 146097 + day_of_era - 719468;

    // The weekday must agree with the date.
    expected := (days + 3) % 7;

    if expected < 0 expected += 7;

    if expected != weekday return 0, false;

    return days * 86400 + hour * 3600 + minute * 60 + second, true;
}

#scope_module

two_digits :: (text: *[29] u8, at: int, value: int) {
    text.*[at] = cast(u8) (#char "0" + value / 10 % 10);
    text.*[at + 1] = cast(u8) (#char "0" + value % 10);
}

read_digits :: (text: string, start: int, count: int) -> int, bool {
    value := 0;

    for start..start + count - 1 {
        if !is_digit(text[it]) return 0, false;

        value = value * 10 + (text[it] - #char "0");
    }

    return value, true;
}

#scope_export
