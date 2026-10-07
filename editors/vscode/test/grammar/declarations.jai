// SYNTAX TEST "source.jai" "declarations"

main :: () {
// <---- entity.name.function.jai
//   ^^ keyword.operator.declaration.constant.jai

}

Vec3 :: struct {
// <---- entity.name.type.jai
//      ^^^^^^ storage.type.jai

    x, y, z: float32;
//  ^ variable.other.declaration.jai
//     ^ variable.other.declaration.jai
//        ^ variable.other.declaration.jai
//         ^ punctuation.separator.colon.jai
//           ^^^^^^^ support.type.builtin.jai

}

Color :: enum u8 { RED; GREEN; }
// <----- entity.name.type.jai
//       ^^^^ storage.type.jai
//            ^^ support.type.builtin.jai

MAX :: 64;
// <--- variable.other.constant.declaration.jai
//     ^^ constant.numeric.integer.decimal.jai

N :: (1 + 2) * 3;
// <- variable.other.constant.declaration.jai

  count := 0;
//^^^^^ variable.other.declaration.jai
//      ^^ keyword.operator.declaration.jai

  a, b := f();
//^ variable.other.declaration.jai
//   ^ variable.other.declaration.jai
//        ^ entity.name.function.call.jai

  nodes: [..] *Node;
//^^^^^ variable.other.declaration.jai
//            ^ keyword.operator.pointer.jai
//             ^^^^ entity.name.type.jai

  x: Vec3 = ---;
//   ^^^^ entity.name.type.jai
//          ^^^ constant.language.uninitialized.jai

add :: inline (a: int, b: int) -> int #c_call {
// <--- entity.name.function.jai
//     ^^^^^^ storage.modifier.jai
//             ^ variable.other.declaration.jai
//                             ^^ keyword.operator.arrow.jai
//                                ^^^ support.type.builtin.jai
//                                    ^^^^^^^ storage.modifier.directive.jai

    return a + b;
//  ^^^^^^ keyword.control.jai

}

make :: (count: int) -> Buffer {
//                      ^^^^^^ entity.name.type.jai

}

callback :: (x: int) #foreign libc "callback";
// <-------- entity.name.function.jai
//                   ^^^^^^^^ storage.modifier.directive.jai

  parse :: (text: string) -> ok: bool, value: int {
//^^^^^ entity.name.function.jai
//                           ^^ variable.other.declaration.jai
//                               ^^^^ support.type.builtin.jai

  }
