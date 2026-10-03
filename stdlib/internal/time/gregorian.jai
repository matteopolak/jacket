valid_date :: (year: s32, month: s32, day: s32) -> bool {
    if month < 0 || month >= 12 || day < 0 return false;

    days := s32.[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    limit := days[month];

    if month == 1 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) limit += 1;

    return day < limit;
}
