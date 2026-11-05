//! The Rust authority for generated JSON Schema and TypeScript consumer declarations.
//! Offsets are UTF-8 byte offsets. IDs and destinations are logical and repository-relative.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! dto {
    ($($name:ident { $($field:ident: $ty:ty),* $(,)? })*) => {$ (
        #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    )*};
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Schema {
    #[serde(rename = "jacket-doc-site")]
    JacketDocSite,
}

/// Changing declaration formatting semantics requires an explicit profile version.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum DeclarationFormatProfile {
    #[serde(rename = "documentation-declarations/1")]
    DocumentationDeclarationsV1,
}

dto! {
    Manifest {
        schema: Schema,
        schema_version: u32,
        producer: Producer,
        source: SourceState,
        partial: bool,
        capabilities: Vec<Capability>,
        shards: Vec<Inventory>,
        assets: Vec<Asset>,
    }
    PageMetadata {
        title: String, description: String, url: Option<String>, canonical: Option<String>,
        og_type: OpenGraphType, image: Option<String>, twitter_card: TwitterCard,
        noindex: bool, structured: Vec<StructuredData>,
    }
    BreadcrumbEntry { position: u64, name: String, url: Option<String> }
    Producer { jacket: String, compiler: String, formatter: Option<Formatter> }
    Formatter { version: String, sha256: String, profile: DeclarationFormatProfile, max_width: u32, indent_width: u32 }
    SourceState {
        revision: Option<String>, dirty: bool, configuration_digest: String, inputs: Vec<InputDigest>,
    }
    InputDigest { path: String, sha256: String }
    Inventory { path: String, sha256: String, bytes: u64 }
    Asset { path: String, media_type: String, sha256: String, bytes: u64, provenance: AssetProvenance }
    SiteData {
        name: String, description: String, version: String, footer_version: String, breadcrumb: String,
        page_base: String, social_site_name: String, canonical_version: Option<String>, social_image: Option<String>, canonical_origin: Option<String>, playground_url: Option<String>,
        logo: Option<String>, repository: Option<String>,
        header_links: Vec<Link>, categories: Vec<Category>, navigation: Vec<NavigationGroup>,
        routes: Vec<PageRoute>,
    }
    Category { id: String, name: String, badge: String, tip: String, color: String, dark_color: String }
    Link { label: String, target: Target }
    NavigationGroup { id: String, label: String, children: Vec<NavigationGroup>, entries: Vec<NavigationEntry> }
    NavigationEntry { id: String, label: String, title: Option<String>, target: Target, experimental: bool, depth: u32 }
    PageRoute {
        id: String, label: String, route: String, output: String, kind: PageKind,
        anchors: Vec<String>, aliases: Vec<String>, private: bool, noindex: bool, shard: String,
    }
    Location { file: String, line: u64 }
    TextSpan { start: u64, end: u64, role: TokenRole, target: Option<Target> }
    ResolvedText { source: String, spans: Vec<TextSpan> }
    ApiModule {
        id: String, title: String, source: Location, source_url: Option<String>, docs: Vec<DocumentNode>,
        implicit: bool, experimental: bool, wasm: bool, since: String,
        platforms: Vec<String>, category: Option<String>, group: String,
        parameters: Vec<Parameter>, declarations: Vec<Declaration>, dependencies: Vec<InputDigest>, metadata: PageMetadata,
    }
    Declaration {
        id: String, name: String, kind: DeclarationKind, route: String,
        source: Location, source_url: Option<String>, private: bool, binding: bool,
        experimental: bool, since: String, platforms: Vec<String>, group: String,
        docs: Vec<DocumentNode>, definitions: Vec<FormattedDefinition>, value: ResolvedText,
        members: Vec<Member>, overloads: Vec<Overload>, metadata: PageMetadata,
    }
    FormattedDefinition { id: String, kind: DeclarationKind, source: Location, source_url: Option<String>, platforms: Vec<String>, scope: Vec<String>, docs: Vec<DocumentNode>, code: ResolvedText, anchors: Vec<DefinitionAnchor> }
    DefinitionAnchor { id: String, name: String, start: u64, end: u64 }
    Parameter { name: String, ty: ResolvedText, default: ResolvedText, docs: Vec<DocumentNode> }
    Member { id: String, name: String, ty: ResolvedText, docs: Vec<DocumentNode> }
    Overload {
        id: String, label: Option<String>, source: Location, signature: ResolvedText,
        docs: Vec<DocumentNode>, parameters: Vec<Parameter>, results: Vec<Parameter>, flags: Vec<String>, platforms: Vec<String>,
    }
    DocumentPage { id: String, title: String, kind: DocumentKind, source: Option<Location>, nodes: Vec<DocumentNode>, outline: Vec<Heading>, metadata: PageMetadata }
    Heading { id: String, label: String, level: u32 }
    Example {
        id: String, language: String, source: ResolvedText, fence: String,
        location: Location, runnable: bool, reason: Option<String>,
        program: Option<String>, run_template: Option<ProgramTemplate>,
        open_template: Option<ProgramTemplate>, playground_url: Option<String>,
        timeout_ms: u64, capabilities: Vec<Capability>,
    }
    ProgramTemplate { head: String, middle: String, tail: String, wrapped: bool, pad: bool }
    SearchEntry {
        id: String, name: String, qualified_name: String, module: Option<String>,
        target: Target, summary: String, kind: SearchKind, binding: bool,
        experimental: bool, category: Option<String>, overloads: Vec<SearchSignature>, preview: Option<SearchPreview>,
    }
    SearchParameter { name: String, ty: String, default: String }
    SearchSignature { parameters: Vec<String>, results: Vec<String> }
    SearchData { schema_version: u32, entries: Vec<SearchEntry> }
    NavigationData { schema_version: u32, module_id: String, entries: Vec<NavigationDeclaration> }
    NavigationDeclaration { id: String, route: String, name: String, kind: DeclarationKind, binding: bool, experimental: bool, private: bool, group: String }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchPreview {
    Procedure {
        parameters: Vec<SearchParameter>,
        results: Vec<SearchParameter>,
    },
    Value {
        value: String,
    },
    DeclarationKind {
        declaration_kind: DeclarationKind,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Page { id: String, anchor: Option<String> },
    External { url: String },
    Asset { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentNode {
    Text {
        text: String,
    },
    Paragraph {
        children: Vec<Self>,
    },
    Heading {
        id: Option<String>,
        level: u32,
        children: Vec<Self>,
    },
    Emphasis {
        children: Vec<Self>,
    },
    Strong {
        children: Vec<Self>,
    },
    Strikethrough {
        children: Vec<Self>,
    },
    Code {
        text: String,
    },
    Link {
        target: Target,
        title: String,
        children: Vec<Self>,
    },
    Image {
        target: Target,
        title: String,
        alternative: String,
    },
    List {
        start: Option<u64>,
        children: Vec<Self>,
    },
    ListItem {
        children: Vec<Self>,
    },
    Quote {
        children: Vec<Self>,
    },
    Table {
        alignments: Vec<Alignment>,
        head: Vec<Self>,
        rows: Vec<Self>,
    },
    TableRow {
        cells: Vec<Self>,
    },
    TableCell {
        children: Vec<Self>,
    },
    Break {
        hard: bool,
    },
    Rule {},
    CodeBlock {
        source: ResolvedText,
        language: String,
        info: String,
    },
    Fence {
        example: Box<Example>,
    },
    ExampleGroup {
        title: ExampleTitle,
        note: Option<String>,
        source: Box<Self>,
        result: Option<ExampleResult>,
    },
    Compare {
        before: ResolvedText,
        after: ResolvedText,
        settings: String,
    },
}

/// A generated example owns its optional result; ordinary authored fences remain independent.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExampleResult {
    pub kind: ExampleResultKind,
    pub body: Box<DocumentNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StructuredData {
    TechArticle {
        headline: String,
        description: String,
        url: Option<String>,
        website_name: String,
        website_url: Option<String>,
        module_name: String,
    },
    BreadcrumbList {
        entries: Vec<BreadcrumbEntry>,
    },
}

macro_rules! enumeration {
    ($($name:ident { $($variant:ident),* $(,)? })*) => {$ (
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),* }
    )*};
}

enumeration! {
    ExampleTitle { Example, Fixed }
    ExampleResultKind { Diagnostic, Output }
    OpenGraphType { Website, Article }
    TwitterCard { Summary, SummaryLargeImage }
    DocumentKind { Home, NotFound, Guide, Tool }
    PageKind { Home, NotFound, Module, Declaration, Guide, Tool }
    DeclarationKind { Procedure, Struct, Union, Enum, EnumFlags, Type, Constant }
    SearchKind { Module, Declaration, Guide, Tool }
    TokenRole { Plain, Keyword, Type, String, Number, Comment, Directive, DeclaredName }
    Alignment { None, Left, Center, Right }
    Capability { BrowserRuntime, EditableExamples }
    AssetProvenance { ConfiguredLogo, ConfiguredImage, BrowserRuntime, GeneratedMetadata }
}

/// All payload shapes used to generate the one consumer schema without hand-maintained types.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "payload",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Payload {
    Manifest(Manifest),
    Site(SiteData),
    Module(ApiModule),
    Document(DocumentPage),
    Search(SearchData),
    Example(Example),
    Navigation(NavigationData),
}

impl Manifest {
    /// Validate the envelope before opening any inventoried file.
    /// For example, `../site.json` is rejected even if its digest is well formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "/schemaVersion: unsupported jacket-doc-site version {}; expected 1",
                self.schema_version
            ));
        }
        if let Some(formatter) = &self.producer.formatter {
            validate_digest(&formatter.sha256)
                .map_err(|error| format!("/producer/formatter/sha256: {error}"))?;
            if formatter.version.trim().is_empty()
                || formatter.max_width == 0
                || formatter.indent_width == 0
            {
                return Err(
                    "/producer/formatter: version and nonzero layout options are required".into(),
                );
            }
        }
        let runtime = self.capabilities.contains(&Capability::BrowserRuntime);
        if runtime != self.capabilities.contains(&Capability::EditableExamples)
            || self
                .capabilities
                .iter()
                .enumerate()
                .any(|(i, cap)| self.capabilities[..i].contains(cap))
        {
            return Err("/capabilities: browser runtime and editable examples must occur together without duplicates".into());
        }
        let runtime_assets = self
            .assets
            .iter()
            .filter(|asset| {
                asset.provenance == AssetProvenance::BrowserRuntime
                    || asset.path.starts_with("runtime/")
            })
            .collect::<Vec<_>>();
        if runtime {
            if runtime_assets.len() != 2
                || ![
                    ("runtime/engine.mjs", "text/javascript"),
                    ("runtime/jai_wasm.wasm", "application/wasm"),
                ]
                .iter()
                .all(|(path, media)| {
                    runtime_assets.iter().any(|asset| {
                        asset.path == *path
                            && asset.media_type == *media
                            && asset.provenance == AssetProvenance::BrowserRuntime
                    })
                })
            {
                return Err("/assets: browser runtime capability requires exactly the engine and Wasm runtime assets with matching provenance and media types".into());
            }
        } else if !runtime_assets.is_empty() {
            return Err("/assets: runtime assets require the browser runtime capability".into());
        }
        let mut paths = std::collections::BTreeSet::new();
        for (i, entry) in self.shards.iter().enumerate() {
            validate_path(&entry.path).map_err(|e| format!("/shards/{i}/path: {e}"))?;
            validate_digest(&entry.sha256).map_err(|e| format!("/shards/{i}/sha256: {e}"))?;
            if !paths.insert(entry.path.to_lowercase()) {
                return Err(format!(
                    "/shards/{i}/path: duplicate case-folded destination {}",
                    entry.path
                ));
            }
        }
        for (i, entry) in self.assets.iter().enumerate() {
            validate_path(&entry.path).map_err(|e| format!("/assets/{i}/path: {e}"))?;
            validate_digest(&entry.sha256).map_err(|e| format!("/assets/{i}/sha256: {e}"))?;
            if !paths.insert(entry.path.to_lowercase()) {
                return Err(format!(
                    "/assets/{i}/path: duplicate case-folded destination {}",
                    entry.path
                ));
            }
        }
        validate_digest(&self.source.configuration_digest)
            .map_err(|e| format!("/source/configurationDigest: {e}"))?;
        for (i, input) in self.source.inputs.iter().enumerate() {
            validate_path(&input.path).map_err(|e| format!("/source/inputs/{i}/path: {e}"))?;
            validate_digest(&input.sha256)
                .map_err(|e| format!("/source/inputs/{i}/sha256: {e}"))?;
        }
        Ok(())
    }
}

/// Portable export destinations never contain native roots or traversal components.
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains(['\\', ':', '\0', '<', '>', '"', '|', '?', '*'])
        || path.split('/').any(|p| {
            matches!(p, "" | "." | "..") || p.ends_with([' ', '.']) || {
                let stem = p.split('.').next().unwrap_or_default().to_ascii_uppercase();
                matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (stem.len() == 4
                        && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && matches!(stem.as_bytes()[3], b'1'..=b'9'))
            }
        })
    {
        return Err(format!("unsafe relative path {path:?}"));
    }
    Ok(())
}

fn validate_digest(digest: &str) -> Result<(), String> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("expected a lowercase SHA-256 digest".into());
    }
    Ok(())
}

impl From<DocumentKind> for PageKind {
    fn from(kind: DocumentKind) -> Self {
        match kind {
            DocumentKind::Home => Self::Home,
            DocumentKind::NotFound => Self::NotFound,
            DocumentKind::Guide => Self::Guide,
            DocumentKind::Tool => Self::Tool,
        }
    }
}
