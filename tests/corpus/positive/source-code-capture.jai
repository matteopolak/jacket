main :: () -> int { x:=3; body::#code { x += 2; }; { x:=100; #insert body; } return x; }
