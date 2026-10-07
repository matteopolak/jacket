// SYNTAX TEST "source.jai" "statements, directives, comments"

#import "Basic";
// <------- keyword.control.import.directive.jai
//      ^^^^^^^ string.quoted.double.jai

#load "other.jai";
// <----- keyword.control.import.directive.jai

/* outer /* nested */ still a comment */ x := 1;
// <-- comment.block.jai punctuation.definition.comment.begin.jai
//       ^^ comment.block.jai comment.block.jai punctuation.definition.comment.begin.jai
//                    ^^^^^ comment.block.jai
//                                       ^ variable.other.declaration.jai

  #run main();
//^^^^ keyword.other.directive.jai

  #if OS == .WINDOWS {
//^^^ keyword.control.conditional.directive.jai
//          ^ punctuation.accessor.jai
//           ^^^^^^^ variable.other.member.jai

  }

  #insert #code { x += 1; };
//^^^^^^^ keyword.other.directive.jai
//        ^^^^^ keyword.other.directive.jai

  sum :: (values: [] $T, $$scale: T) -> T #expand {
//                   ^ punctuation.definition.polymorph.jai
//                    ^ entity.name.type.parameter.jai
//                       ^^ punctuation.definition.polymorph.jai
//                         ^^^^^ variable.parameter.baked.jai
//                                        ^^^^^^^ storage.modifier.directive.jai

  }

  show :: (thing: $R/Ring) #modify { return true; } {}
//                ^ punctuation.definition.polymorph.jai
//                 ^ entity.name.type.parameter.jai
//                         ^^^^^^^ storage.modifier.directive.jai

  for item, index: items { print("%\n", it_index); }
//^^^ keyword.control.loop.jai
//    ^^^^ variable.other.declaration.jai
//          ^^^^^ variable.other.declaration.jai
//                 ^^^^^ - entity.name.type.jai
//                                      ^^^^^^^^ variable.language.it.jai

  for < *v: list if it > 0 then continue;
//    ^ keyword.operator.reverse.jai
//      ^ keyword.operator.pointer.jai
//       ^ variable.other.declaration.jai
//                  ^^ variable.language.it.jai

  for values total += it;
//    ^^^^^^ - variable.other.declaration.jai
//                 ^^ keyword.operator.assignment.compound.jai

  y := cast(u8) x + cast,no_check(s32) z + xx,trunc w;
//     ^^^^ keyword.operator.cast.jai
//                  ^^^^ keyword.operator.cast.jai
//                      ^^^^^^^^^ storage.modifier.cast.jai
//                                         ^^ keyword.operator.cast.jai
//                                           ^^^^^^ storage.modifier.cast.jai

  v := ifx a > b then a else b;
//     ^^^ keyword.control.jai
//               ^^^^ keyword.control.jai
//                      ^^^^ keyword.control.jai

  using context.allocator;
//^^^^^ keyword.other.using.jai
//      ^^^^^^^ variable.language.context.jai

  defer free(buffer);
//^^^^^ keyword.control.jai
//      ^^^^ entity.name.function.call.jai

  push_context new_context { work(); }
//^^^^^^^^^^^^ keyword.control.jai

  size := size_of(Vec3) + type_info(int).runtime_size;
//        ^^^^^^^ support.function.builtin.jai
//                        ^^^^^^^^^ support.function.builtin.jai

  Thing :: struct { value: int; } @Serialize
//                                ^^^^^^^^^^ storage.modifier.note.jai

  ptr.* = ptr2.*;
//   ^^ keyword.operator.dereference.jai

  if x == {

    case .A; #through;
//  ^^^^ keyword.control.jai
//           ^^^^^^^^ keyword.control.conditional.directive.jai

  }

  // a line comment with "quotes"
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ comment.line.double-slash.jai

  `x = 1;
//^ keyword.operator.backtick.jai
