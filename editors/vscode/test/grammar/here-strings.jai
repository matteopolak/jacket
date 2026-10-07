// SYNTAX TEST "source.jai" "language-tagged here-strings"

SHADER :: #string,cr WGSL
//        ^^^^^^^ keyword.other.directive.here-string.jai
//               ^^^ storage.modifier.here-string.jai
//                   ^^^^ entity.name.tag.here-string.jai
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
// <------------------------------------------------------------------------ meta.embedded.block.wgsl
// <- punctuation.definition.attribute.wgsl
//^^^^^ entity.name.function.decorator.wgsl
//      ^^ storage.type.function.wgsl
//         ^^ entity.name.function.wgsl
//                                      ^^^ support.type.wgsl
//                                                                 ^^^^^ support.type.wgsl
    var<storage, read_write> data: array<f32>; /* left open
//  ^^^ storage.type.variable.wgsl
//      ^^^^^^^ storage.modifier.wgsl
//               ^^^^^^^^^^ storage.modifier.wgsl
//                                             ^^ comment.block.wgsl
    return vec4f(1.0, 0.5h, 2u, 0x1F);
//  ^^^^^^ comment.block.wgsl
    WGSL;
//  ^^^^ entity.name.tag.here-string.jai
//  ^^^^ - meta.embedded.block.wgsl
//      ^ punctuation.terminator.statement.jai
//      ^ - comment

after := 1;
// <----- variable.other.declaration.jai
// <----- - meta.here-string.wgsl.jai

QUERY :: #string sql
//               ^^^ entity.name.tag.here-string.jai
SELECT name FROM users
// <---------------------- meta.embedded.block.sql
// <------ keyword.other.sql
//          ^^^^ keyword.other.sql
SQL is the tag spelled differently, so not the terminator
// <---------------------------------------------------------- meta.embedded.block.sql
sql_table is a longer identifier
// <-------------------------------- meta.embedded.block.sql
sql
// <--- entity.name.tag.here-string.jai

// No GLSL grammar is installed here: the tag still matches (ignoring case), but with nothing to
// highlight the body with, the rule drops out and the body stays a plain string.
LIGHT :: #string Glsl
void main() {}
// <-------------- string.unquoted.here-string.body.jai
Glsl
// <---- entity.name.tag.here-string.jai

DATA :: #string JSON
{ "a": 1 }
// <---------- meta.embedded.block.json
//^^^ string.quoted.double.json
JSON

INNER :: #string JAI
x := 2;
// <- meta.embedded.block.jai variable.other.declaration.jai
JAI

TEXT :: #string END
fn not_wgsl() {}
// <---------------- string.unquoted.here-string.body.jai
// <---------------- - meta.embedded
END
// <--- entity.name.tag.here-string.jai

OTHER :: #string WGSL_SOURCE
fn not_wgsl() {}
// <---------------- string.unquoted.here-string.body.jai
WGSL_SOURCE

last := 3;
// <---- variable.other.declaration.jai
