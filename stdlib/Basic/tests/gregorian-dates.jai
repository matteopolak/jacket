// Exercise the exact production validation kernel without a fake Windows API.
#import "Basic";
#load "../../internal/time/gregorian.jai";

main :: () {
    assert(valid_date(2000, 1, 28));
    assert(!valid_date(1900, 1, 28));
    assert(valid_date(2400, 1, 28));
    assert(!valid_date(2001, 1, 28));
    assert(!valid_date(2000, -1, 0) && !valid_date(2000, 12, 0));
    assert(!valid_date(2000, 0, -1) && !valid_date(2000, 0, 31));
    assert(valid_date(2000, 0, 30) && !valid_date(2000, 3, 30));
    total: s64;

    for year: 2000..2399 {
        for month: 0..11 {
            days: s64;
            rejected := false;

            for day: 0..31 {
                accepted := valid_date(cast(s32) year, cast(s32) month, cast(s32) day);
                if accepted {
                    assert(!rejected);
                    days += 1;
                } else rejected = true;
            }

            assert(days >= 28 && days <= 31);
            total += days;
        }
    }

    assert(total == 146097);
    print("ok\n");
}
