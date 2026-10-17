#import "Basic";

Point :: struct {
  x: float;
        y: float;
}

main :: () {
  total := 0;
  for i: 0..9 {
          if i % 2 == 0 {
      total += i;
          }
  }
  // A call continued over several lines keeps its alignment.
    result := add(total,
                  10,
                  20);
  if total > 10 &&
     result > 0 {
      print("big\n");
  }
  callback := (x: int) -> int {
          return x * 2;
  };
  values := int.[
      1, 2, 3,
  ];
  if total > 3
      print("braceless body keeps its offset\n");
}

add :: (a: int,
        b: int,
        c: int) -> int {
        return a + b + c;
}
