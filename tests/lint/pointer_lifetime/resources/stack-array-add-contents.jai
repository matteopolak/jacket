#import "Basic";
Node::struct {value:s64;}

overwritten::(p:*Node) {
    backing:[4]s64;
    (cast(**Node)*backing).*=p;
    free(p);
    values:[..]s64;values.data=cast(*s64)*backing;values.allocated=4;
    slot:=array_add(*values);
    loaded:=(cast(**Node)*backing).*;
    print("%\n",loaded.value);
}

captured::(p:*Node) {
    backing:[4]s64;
    (cast(**Node)*backing).*=p;
    saved:=(cast(**Node)*backing).*;
    free(saved);
    values:[..]s64;values.data=cast(*s64)*backing;values.allocated=4;
    slot:=array_add(*values);
    print("%\n",saved.value);
}

unrelated::(p:*Node) {
    saved:=p;free(saved);
    backing:[4]s64;values:[..]s64;values.data=cast(*s64)*backing;values.allocated=4;
    slot:=array_add(*values);
    print("%\n",saved.value);
}

main::(){}
