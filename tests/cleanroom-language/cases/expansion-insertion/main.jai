#import "Basic";

increase_balance :: (amount: s64) #expand {
    `balance += amount;
}

main :: () {
    balance := 6;
    increase_balance(7);
    #insert "balance *= 2;";
    assert(balance == 26, "expansion and inserted text resolve at the call site");
    print("ok\n");
}
