touch :: (counter: *int) {
    counter.* += 1;
}

exercise :: (flag: bool) -> int {
    counter: int;

    ifx flag {
        touch(*counter);
    } else {
        touch(*counter);
        touch(*counter);
    }

    return counter;
}

#assert exercise(true) == 1;
#assert exercise(false) == 2;

deferred :: (flag: bool) -> int {
    counter: int;

    ifx flag {
        defer touch(*counter);
        touch(*counter);
    } else {
        ifx !flag {
            touch(*counter);
        } else {
            touch(*counter);
            touch(*counter);
        }
    }

    return counter;
}

omitted :: () -> int {
    counter: int;

    ifx false {
        touch(*counter);
    }

    return counter;
}

#assert deferred(true) == 2;
#assert deferred(false) == 1;
#assert omitted() == 0;

main :: () -> int {
    if exercise(true) != 1 return 1;
    if exercise(false) != 2 return 2;
    if deferred(true) != 2 return 3;
    if deferred(false) != 1 return 4;
    if omitted() != 0 return 5;

    return 0;
}
