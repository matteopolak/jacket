// Written ordinary argument lists select nominal instances, even when defaults match.
Omitted :: #import "Leaf"(FLAG = true);
Explicit :: #import "Leaf"(FLAG = true, SECOND = true);
Again :: #import "Leaf"(FLAG = true, SECOND = true);
Reordered :: #import "Leaf"(SECOND = true, FLAG = true);
Plain :: #import "Leaf";
Different :: #import "Leaf"(FLAG = false, SECOND = true);
Bridge :: #import "Bridge";
Configured_Bridge :: #import "Bridge"(FLAG = true);

#assert Omitted.both && Explicit.both && Reordered.both && Plain.both;
#assert !Different.both;
#assert type_of(Bridge.native) == *Explicit.Token;
#assert type_of(Configured_Bridge.native) == *Explicit.Token;
#assert type_of(Bridge.native) != *Omitted.Token;
#assert Explicit.Token == Again.Token;
#assert Explicit.Token != Omitted.Token;
#assert Explicit.Token != Reordered.Token;
#assert Explicit.Token != Plain.Token;
#assert Explicit.Token != Different.Token;
#assert *Explicit.state == *Again.state;
#assert *Explicit.state != *Omitted.state;
#assert *Explicit.state != *Reordered.state;
#assert *Explicit.state != *Plain.state;

storage :: () -> bool #no_context {
    Explicit.state = 19;
    if Again.state != 19 || Omitted.state != 0 || Reordered.state != 0 || Plain.state != 0 return false;
    Omitted.state = 23;
    return Explicit.state == 19 && Again.state == 19 && Omitted.state == 23;
}

#assert storage();
main :: () {}
