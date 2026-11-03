// The example takes the file's settings except those `[doc.format]` overrides: no blank line
// after the declaration, and three-space indent. Return keeps its blank line.
//
// ```jai
// total:=0;
// step:=2;
// print("%\n",total);
// print("%\n",step);
// if total<1{
// total=1;
// }
// return;
// ```
f::(){
a:=1;
b:=2;
print("%\n",a+b);
print("%\n",a);
print("%\n",b);
}
