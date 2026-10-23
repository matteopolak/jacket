valid_date :: (year: s32, month: s32, day: s32) -> bool {
    if month < 0 || month >= MONTHS_PER_YEAR || day < 0 return false;

    days := s32.[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    limit := days[month];

    if month == 1 && year % 4 == 0 && (year % 100 != 0 || year % GREGORIAN_CYCLE_YEARS == 0) limit += 1;

    return day < limit;
}

#scope_file
MONTHS_PER_YEAR :: 12;
GREGORIAN_CYCLE_YEARS :: 400;
