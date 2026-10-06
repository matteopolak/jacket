Math :: #import "Math";
#import "Math";
GETRECT_Y_UP :: true;
#load "../../../stdlib/GetRect_Common/geometry.jai";
#load "../../../stdlib/GetRect_Common/geometry-helpers.jai";
rectangle_cases :: () -> bool {
    if size_of(Rect)!=16 return false;
    r:=get_rect(x=10,y=20,w=100,h=60);
    top,top_remainder:=cut_top(rect=r,amount=15,margin=2);
    if top.x!=10 || top.y!=64 || top.w!=100 || top.h!=15 return false;
    if top_remainder.y!=20 || top_remainder.h!=43 return false;
    bottom,bottom_remainder:=cut_bottom(r,15,2);
    if bottom.y!=20 || bottom.h!=15 || bottom_remainder.y!=37 || bottom_remainder.h!=43 return false;
    left,left_remainder:=cut_left(r,20,3);
    right,right_remainder:=cut_right(r,20,3);
    if left.x!=10 || left.w!=20 || left_remainder.x!=33 || left_remainder.w!=77 return false;
    if right.x!=90 || right.w!=20 || right_remainder.x!=10 || right_remainder.w!=77 return false;
    grown:=expand(r,4); recovered:=shrink(grown,4);
    if recovered.x!=r.x || recovered.y!=r.y || recovered.w!=r.w || recovered.h!=r.h return false;
    narrowed:=shrink_x(r,3); shorter:=shrink_y(r,4);
    if narrowed.x!=13 || narrowed.w!=94 || shorter.y!=24 || shorter.h!=52 return false;
    if !is_inside(10,20,r) || is_inside(110,20,r) || is_inside(10,80,r) return false;
    clipped:=intersect(r,get_rect(90,50,40,50));
    if clipped.x!=90 || clipped.y!=50 || clipped.w!=20 || clipped.h!=30 return false;
    disjoint:=intersect(r,get_rect(300,300,2,2));
    if disjoint.w!=0 || disjoint.h!=0 return false;
    envelope:=bound(r,get_rect(0,10,20,20));
    if envelope.x!=0 || envelope.y!=10 || envelope.w!=110 || envelope.h!=70 return false;
    p0,p1,p2,p3:=get_quad(get_rect(0.2,1.2,2.2,3.2),snap_coordinates=true);
    if p0.x!=0 || p0.y!=1 || p1.x!=2 || p1.y!=1 || p2.x!=2 || p2.y!=4 || p3.x!=0 || p3.y!=4 return false;
    return true;
}
color_cases :: () -> bool {
    original:=xyzw(0.2,0.4,0.8,0.6);
    dim:=darken(original,0.5);
    pale:=whiten(original,0.25);
    lifted:=brighten(original,1);
    if abs(dim.x-0.1)>0.000001 || abs(dim.z-0.4)>0.000001 || dim.w!=original.w return false;
    if abs(pale.x-0.4)>0.000001 || abs(pale.z-0.85)>0.000001 || pale.w!=original.w return false;
    if abs(lifted.x-0.25)>0.000001 || abs(lifted.z-1)>0.000001 || lifted.w!=original.w return false;
    r,g,b:=hsv_to_rgb(0,1,1);
    if r!=1 || g!=0 || b!=0 return false;
    h,s,v:=rgb_to_hsv(0.25,0.5,0.75);
    r2,g2,b2:=hsv_to_rgb(h,s,v);
    if abs(r2-0.25)>0.000001 || abs(g2-0.5)>0.000001 || abs(b2-0.75)>0.000001 return false;
    if move_toward(a=0,b=10,amount_increasing=3)!=3 return false;
    if move_toward(a=10,b=0,amount_increasing=3,amount_decreasing=2)!=8 return false;
    value:float=1;
    move_toward(*value,target=0,dt=0.5,rate_up=4,rate_down=1);
    if value!=0.5 || fract(-0.25)!=0.75 return false;
    return true;
}
hash_cases :: () -> bool {
    loc:=Source_Code_Location.{fully_pathed_filename="/geometry-fixture.jai",line_number=7};
    first:=get_hash(loc,7);
    if get_hash(loc,7)!=first || get_hash(loc,8)==first return false;
    if combine_hashes(0,0)!=0 || combine_hashes(1,0)!=KNUTH_GOLDEN_RATIO_64 return false;
    return true;
}
#assert #run rectangle_cases();
#assert #run color_cases();
#assert #run hash_cases();
