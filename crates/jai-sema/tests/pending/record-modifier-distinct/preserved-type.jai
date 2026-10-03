First::#type,distinct u8; initial:First:21;
Box::struct(T:Type,Fill:T) #modify {return true;} {value:T=Fill;}
Ready::#type Box(First,initial); saved:Ready;
main::()->int{return cast(int)saved.value;}
