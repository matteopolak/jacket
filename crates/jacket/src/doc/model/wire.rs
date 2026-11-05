//! Strict compiler wire DTOs. Defaults are limited to fields the producer omits when empty.
use super::Kind;
use serde::Deserialize;

/// References always carry declaration identity and visibility in four columns.
pub(super) type TypeRef = (String, String, usize, bool);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Module {
    pub schema: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub file: String,
    pub implicit: bool,
    pub docs: String,
    pub docs_line: usize,
    pub platforms: Vec<String>,
    pub experimental: bool,
    pub parameters: Vec<Param>,
    pub items: Vec<Item>,
    pub unresolved: Vec<Unresolved>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub default: String,
    pub doc: String,
    #[serde(default)]
    pub refs: Vec<TypeRef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Item {
    pub name: String,
    pub kind: Kind,
    pub file: String,
    pub line: usize,
    pub docs: String,
    pub value: String,
    pub platforms: Vec<String>,
    pub experimental: bool,
    pub overloads: Vec<Overload>,
    pub members: Vec<Member>,
    pub scope: Vec<String>,
    #[serde(default)]
    pub binding: bool,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub definitions: Vec<Definition>,
    #[serde(default)]
    pub also: Vec<(String, usize)>,
    #[serde(default)]
    pub refs: Vec<TypeRef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Overload {
    pub file: String,
    pub line: usize,
    pub signature: String,
    pub params: Vec<Param>,
    pub returns: Vec<Param>,
    pub docs: String,
    pub docs_line: usize,
    pub flags: Vec<String>,
    pub platforms: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Member {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub doc: String,
    #[serde(default)]
    pub refs: Vec<TypeRef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Unresolved {
    pub file: String,
    pub line: usize,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dependency {
    pub file: String,
    pub sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Definition {
    pub kind: Kind,
    pub docs: String,
    pub file: String,
    pub line: usize,
    pub start: u32,
    pub end: u32,
    pub platforms: Vec<String>,
    pub guards: Vec<CompileGuard>,
    pub scope: Vec<String>,
    pub tokens: Vec<DeclarationToken>,
    pub members: Vec<Member>,
    #[serde(default)]
    pub refs: Vec<TypeRef>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompileGuard {
    pub file: String,
    pub start: u32,
    pub end: u32,
    pub tokens: Vec<String>,
    pub taken: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclarationToken {
    pub text: String,
    pub start: u32,
    pub end: u32,
    // Validated compiler facts consumed only by other raw-model clients; layout uses jaifmt.
    #[serde(rename = "statement")]
    pub _statement: bool,
    pub reference: bool,
    pub names: Vec<String>,
    #[serde(rename = "docs")]
    pub _docs: String,
}
