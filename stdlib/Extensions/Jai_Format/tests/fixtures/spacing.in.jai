f :: (a:int,b : *int,c:=3)->int{
    x:=a+b.*;
    y : int = x*2+1;
    z := x-1;
    w := x -1;
    neg := - x;
    not := ! true;
    p := * x;
    q := cast(*u8) *x;
    r := cast,no_check(u8) x;
    s := xx,no_check x;
    t := <<p;
    arr : [4] *int;
    list := int.[ 1 ,2,3 ];
    lit := Vec.{x=1,y=2};
    call(a,b ,c) ;
    if(x==1) return 0;
    while(x>0) x-=1;
    k := (x<<2) | (x>>1);
    m := ifx x>0 then -1 else 1;
    for < i: 0..x-1 {}
    for *it: arr {}
    u := (x) * 2;
    v := arr[0] * 2;
    return x+y;
}
Distinct :: #type,isa u32;
kernel :: #library,system "kernel32";
raylib :: #foreign_library,no_dll "lib/raylib";
Handle :: #type, distinct u64;
where :: () { here := #location(code); report(#caller_location,z); }
glCullFace: (mode: GLenum) #c_call ;
