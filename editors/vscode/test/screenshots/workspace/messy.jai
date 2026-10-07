#import "Basic";
Point::struct{x,y:float;}
length_squared::(p:Point)->float{return p.x*p.x+p.y*p.y;}
main::(){
  points:=Point.[.{1,2},.{3,4}];
      for points {
    if length_squared(it)>4 print("% is far\n",it);
  }
}
