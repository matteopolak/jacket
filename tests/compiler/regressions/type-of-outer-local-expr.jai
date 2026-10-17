#import "Basic";
#import "Reflection";

Color :: enum {
    RED;
    GREEN;
}

main :: () {
    assert(enum_value_to_name(Color.GREEN) == "GREEN");
    info := type_info(Color);
    #assert type_of(info.*) == Type_Info_Enum;
    print("ok\n");
}
