#import "Basic";

Record :: struct {
    value: int;
}

verify :: () -> bool {
    seed_code :: #code seed := 2;
    alias_code :: #code Alias :: Record;
    object_code :: #code using object := Record.{ value = 4 };

    forward_code :: #code forward :: (amount: int = 3) -> int {
        return amount + 1;
    }

    callback_code :: #code callback := forward;

    #insert seed_code;
    #insert alias_code;
    #insert object_code;
    #insert forward_code;
    #insert callback_code;
    value = 9;
    #assert Alias == Record;
    if seed != 2 || object.value != 9 || callback() != 4 || callback(amount = 5) != 6 return false;

    source := 7;
    captured :: #code result := source;

    {
        source := 11;
        #insert captured;
        if result != 7 return false;
    }

    {
        source := 11;
        #insert,scope() captured;
        if result != 11 return false;
    }

    sum := 0;

    nested :: #code {
        {
            hidden := 17;
        }

        local := 3;

        defer sum += local;
        sum = 1;
    }

    #insert nested;

    return sum == 4;
}

main :: () -> s32 {
    if !verify() return 1;
    print("ok\n");
    return 0;
}
