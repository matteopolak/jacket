// SYNTAX TEST "source.jai" "literals"

  a := 0x_FF_FF + 0b1010_0101 + 0h3f80_0000;
//     ^^^^^^^^ constant.numeric.integer.hexadecimal.jai
//                ^^^^^^^^^^^ constant.numeric.integer.binary.jai
//                              ^^^^^^^^^^^ constant.numeric.float.hexadecimal.jai

  b := 1_000_000 + 1.5 + 2.5e-3 + 1e9 + .5 + 3.;
//     ^^^^^^^^^ constant.numeric.integer.decimal.jai
//                 ^^^ constant.numeric.float.decimal.jai
//                       ^^^^^^ constant.numeric.float.decimal.jai
//                                ^^^ constant.numeric.float.decimal.jai
//                  ^^ constant.numeric.float.decimal.jai
//                                           ^^ constant.numeric.float.decimal.jai

  for 0..10 {}
//    ^ constant.numeric.integer.decimal.jai
//     ^^ keyword.operator.range.jai
//       ^^ constant.numeric.integer.decimal.jai

  p := v.x + 1.0;
//       ^ variable.other.member.jai
//           ^^^ constant.numeric.float.decimal.jai

  s := "tab\t \x41 \u00e9 \d065 \% \q";
//     ^ string.quoted.double.jai punctuation.definition.string.begin.jai
//         ^^ constant.character.escape.jai
//            ^^^^ constant.character.escape.jai
//                 ^^^^^^ constant.character.escape.jai
//                        ^^^^^ constant.character.escape.jai
//                              ^^ constant.character.escape.jai
//                                 ^^ invalid.illegal.unknown-escape.jai

  c := #char "\n";
//     ^^^^^ keyword.other.directive.char.jai
//           ^^^^ string.quoted.double.char.jai

  t := true && !false || x == null;
//     ^^^^ constant.language.jai
//              ^^^^^ constant.language.jai
//                            ^^^^ constant.language.jai
//          ^^ keyword.operator.logical.jai

  text := #string DONE
//        ^^^^^^^ keyword.other.directive.here-string.jai
//                ^^^^ entity.name.tag.here-string.jai
  code := "not a string start"; // not a comment
// <--------------------------------------------- string.unquoted.here-string.body.jai
  DONE;
//^^^^ entity.name.tag.here-string.jai
//    ^ punctuation.terminator.statement.jai

  crlf := #string,cr END
//        ^^^^^^^ keyword.other.directive.here-string.jai
//               ^^^ storage.modifier.here-string.jai
//                   ^^^ entity.name.tag.here-string.jai
ENDING is not the terminator
// <------ string.unquoted.here-string.body.jai
END
// <--- entity.name.tag.here-string.jai
