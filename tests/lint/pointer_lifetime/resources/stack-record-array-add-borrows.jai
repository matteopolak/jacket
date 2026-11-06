#import "Basic";
Item::struct {value:s64;}

return_record_slot::()->*Item {
    backing:[4]Item;
    values:[..]Item;
    values.data=cast(*Item)*backing;
    values.allocated=4;
    return array_add(*values);
}

use_record_after_backing_scope::() {
    stale:*Item;
    {
        backing:[4]Item;
        values:[..]Item;
        values.data=cast(*Item)*backing;
        values.allocated=4;
        stale=array_add(*values);
    }
    print("%\n",stale.value);
}

main::(){}
