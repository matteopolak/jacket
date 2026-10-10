// SYNTAX TEST "source.jai" "doc comments: Markdown styling that keeps every marker"

// A **bold** word, *italic*, _italic_ and `code`.
//   ^^^^^^^^ markup.bold.jai
//                  ^^^^^^^^ markup.italic.jai
//                            ^^^^^^^^ markup.italic.jai
//                                         ^^^^^^ markup.inline.raw.string.jai
//            ^^^^ comment.line.double-slash.jai - markup.bold.jai markup.italic.jai
x := 1;
// snake_case_name and 2 * 3 * 4 stay plain.
// ^^^^^^^^^^^^^^^ - markup.italic.jai
//                       ^^^^^ - markup.italic.jai
x := 1;
/// # Heading with `code`
//  ^ punctuation.definition.heading.jai
//    ^^^^^^^ markup.heading.jai
//                  ^^^^ markup.heading.jai markup.inline.raw.string.jai
y := 2;
// - Item with `code`
// ^ punctuation.definition.list.begin.markdown.jai
//              ^^^^ markup.inline.raw.string.jai
//   2. Nested **number**
//   ^^ punctuation.definition.list.begin.markdown.jai
//               ^^^^^^ markup.bold.jai
y := 2;
// See [print], [`Basic.print`], [Basic.print()] and [the text](Thing.field), not a[i] or [0].
//      ^^^^^ markup.underline.link.jai
//               ^^^^^^^^^^^^^ markup.underline.link.jai
//                                ^^^^^^^^^^^^^ markup.underline.link.jai
//                                                    ^^^^^^^^ markup.underline.link.jai
//                                                              ^^^^^^^^^^^ markup.underline.link.jai
//                                                                                 ^^^ - markup.underline.link.jai
//                                                                                        ^^^ - markup.underline.link.jai
z := 3;
// A fenced block with no language is Jai, the `//` prefix stays a comment:
//
// ```
// p := Point.{ 1, 2 }; // trailing
//   ^^ keyword.operator.declaration.jai
//                      ^^^^^^^^^^^ comment.line.double-slash.jai
// for items print("%\n", it);
// <-- comment.line.double-slash.jai punctuation.definition.comment.jai
// ^^^ keyword.control.loop.jai
// ```
//
// after
// <-- comment.line.double-slash.jai - markup.fenced_code.block.jai
w := 4;

/* Block with **bold**
//             ^^^^^^^^ markup.bold.jai
 * ```
 * x := 1;
// ^ variable.other.declaration.jai
 * ```
 * - item
// ^ punctuation.definition.list.begin.markdown.jai
 */
v := 5;
