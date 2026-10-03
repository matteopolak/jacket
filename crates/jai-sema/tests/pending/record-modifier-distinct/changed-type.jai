First::#type,distinct u8; Other::#type,distinct u8; initial:First:21;
Box::struct(T:Type,Fill:T) #modify {T=Other;return true;} {value:T=Fill;}
Rejected::#type Box(First,initial); main::()->int{return 0;}
