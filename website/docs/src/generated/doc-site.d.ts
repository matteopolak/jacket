/* Generated from Rust doc/export/contract.rs. Run npm run generate:types. */

/**
 * All payload shapes used to generate the one consumer schema without hand-maintained types.
 */
export type Payload =
  | {
      data: Manifest;
      payload: "manifest";
    }
  | {
      data: SiteData;
      payload: "site";
    }
  | {
      data: ApiModule;
      payload: "module";
    }
  | {
      data: DocumentPage;
      payload: "document";
    }
  | {
      data: SearchData;
      payload: "search";
    }
  | {
      data: Example;
      payload: "example";
    }
  | {
      data: NavigationData;
      payload: "navigation";
    };
export type AssetProvenance = "configured_logo" | "configured_image" | "browser_runtime" | "generated_metadata";
export type Capability = "browser_runtime" | "editable_examples";
/**
 * Changing declaration formatting semantics requires an explicit profile version.
 */
export type DeclarationFormatProfile = "documentation-declarations/1";
export type Schema = "jacket-doc-site";
export type Target =
  | {
      anchor?: string | null;
      id: string;
      kind: "page";
    }
  | {
      kind: "external";
      url: string;
    }
  | {
      kind: "asset";
      path: string;
    };
export type PageKind = "home" | "not_found" | "module" | "declaration" | "guide" | "tool";
export type DocumentNode =
  | {
      kind: "text";
      text: string;
    }
  | {
      children: DocumentNode[];
      kind: "paragraph";
    }
  | {
      children: DocumentNode[];
      id?: string | null;
      kind: "heading";
      level: number;
    }
  | {
      children: DocumentNode[];
      kind: "emphasis";
    }
  | {
      children: DocumentNode[];
      kind: "strong";
    }
  | {
      children: DocumentNode[];
      kind: "strikethrough";
    }
  | {
      kind: "code";
      text: string;
    }
  | {
      title: string;
      children: DocumentNode[];
      kind: "link";
      target: Target;
    }
  | {
      title: string;
      alternative: string;
      kind: "image";
      target: Target;
    }
  | {
      children: DocumentNode[];
      kind: "list";
      start?: number | null;
    }
  | {
      children: DocumentNode[];
      kind: "list_item";
    }
  | {
      children: DocumentNode[];
      kind: "quote";
    }
  | {
      alignments: Alignment[];
      head: DocumentNode[];
      kind: "table";
      rows: DocumentNode[];
    }
  | {
      cells: DocumentNode[];
      kind: "table_row";
    }
  | {
      children: DocumentNode[];
      kind: "table_cell";
    }
  | {
      hard: boolean;
      kind: "break";
    }
  | {
      kind: "rule";
    }
  | {
      info: string;
      kind: "code_block";
      language: string;
      source: ResolvedText;
    }
  | {
      example: Example;
      kind: "fence";
    }
  | {
      title: ExampleTitle;
      kind: "example_group";
      note?: string | null;
      result?: ExampleResult | null;
      source: DocumentNode;
    }
  | {
      after: ResolvedText;
      before: ResolvedText;
      kind: "compare";
      settings: string;
    };
export type Alignment = "none" | "left" | "center" | "right";
export type TokenRole = "plain" | "keyword" | "type" | "string" | "number" | "comment" | "directive" | "declared_name";
export type ExampleTitle = "example" | "fixed";
export type ExampleResultKind = "diagnostic" | "output";
export type DeclarationKind = "procedure" | "struct" | "union" | "enum" | "enum_flags" | "type" | "constant";
export type OpenGraphType = "website" | "article";
export type StructuredData =
  | {
      description: string;
      headline: string;
      kind: "tech_article";
      module_name: string;
      url?: string | null;
      website_name: string;
      website_url?: string | null;
    }
  | {
      entries: BreadcrumbEntry[];
      kind: "breadcrumb_list";
    };
export type TwitterCard = "summary" | "summary_large_image";
export type DocumentKind = "home" | "not_found" | "guide" | "tool";
export type SearchKind = "module" | "declaration" | "guide" | "tool";
export type SearchPreview =
  | {
      kind: "procedure";
      parameters: SearchParameter[];
      results: SearchParameter[];
    }
  | {
      kind: "value";
      value: string;
    }
  | {
      declaration_kind: DeclarationKind;
      kind: "declaration_kind";
    };

export interface Manifest {
  assets: Asset[];
  capabilities: Capability[];
  partial: boolean;
  producer: Producer;
  schema: Schema;
  schemaVersion: number;
  shards: Inventory[];
  source: SourceState;
}
export interface Asset {
  bytes: number;
  mediaType: string;
  path: string;
  provenance: AssetProvenance;
  sha256: string;
}
export interface Producer {
  compiler: string;
  formatter?: Formatter | null;
  jacket: string;
}
export interface Formatter {
  indentWidth: number;
  maxWidth: number;
  profile: DeclarationFormatProfile;
  sha256: string;
  version: string;
}
export interface Inventory {
  bytes: number;
  path: string;
  sha256: string;
}
export interface SourceState {
  configurationDigest: string;
  dirty: boolean;
  inputs: InputDigest[];
  revision?: string | null;
}
export interface InputDigest {
  path: string;
  sha256: string;
}
export interface SiteData {
  description: string;
  breadcrumb: string;
  canonicalOrigin?: string | null;
  canonicalVersion?: string | null;
  categories: Category[];
  footerVersion: string;
  headerLinks: Link[];
  logo?: string | null;
  name: string;
  navigation: NavigationGroup[];
  pageBase: string;
  playgroundUrl?: string | null;
  repository?: string | null;
  routes: PageRoute[];
  socialImage?: string | null;
  socialSiteName: string;
  version: string;
}
export interface Category {
  badge: string;
  color: string;
  darkColor: string;
  id: string;
  name: string;
  tip: string;
}
export interface Link {
  label: string;
  target: Target;
}
export interface NavigationGroup {
  children: NavigationGroup[];
  entries: NavigationEntry[];
  id: string;
  label: string;
}
export interface NavigationEntry {
  title?: string | null;
  depth: number;
  experimental: boolean;
  id: string;
  label: string;
  target: Target;
}
export interface PageRoute {
  aliases: string[];
  anchors: string[];
  id: string;
  kind: PageKind;
  label: string;
  noindex: boolean;
  output: string;
  private: boolean;
  route: string;
  shard: string;
}
export interface ApiModule {
  title: string;
  category?: string | null;
  declarations: Declaration[];
  dependencies: InputDigest[];
  docs: DocumentNode[];
  experimental: boolean;
  group: string;
  id: string;
  implicit: boolean;
  importPath: string;
  metadata: PageMetadata;
  parameters: Parameter[];
  platforms: string[];
  since: string;
  source: Location;
  sourceUrl?: string | null;
  wasm: boolean;
}
export interface Declaration {
  binding: boolean;
  docs: DocumentNode[];
  experimental: boolean;
  group: string;
  id: string;
  kind: DeclarationKind;
  members: Member[];
  metadata: PageMetadata;
  name: string;
  overloads: Overload[];
  platforms: string[];
  private: boolean;
  route: string;
  since: string;
  source: Location;
  sourceUrl?: string | null;
  value: ResolvedText;
  definitions: FormattedDefinition[];
}
export interface ResolvedText {
  source: string;
  spans: TextSpan[];
}
export interface TextSpan {
  end: number;
  role: TokenRole;
  start: number;
  target?: Target | null;
}
export interface Example {
  capabilities: Capability[];
  fence: string;
  id: string;
  language: string;
  location: Location;
  openTemplate?: ProgramTemplate | null;
  playgroundUrl?: string | null;
  program?: string | null;
  reason?: string | null;
  runTemplate?: ProgramTemplate | null;
  runnable: boolean;
  source: ResolvedText;
  timeoutMs: number;
}
export interface Location {
  file: string;
  line: number;
}
export interface ProgramTemplate {
  head: string;
  middle: string;
  pad: boolean;
  tail: string;
  wrapped: boolean;
}
/**
 * A generated example owns its optional result; ordinary authored fences remain independent.
 */
export interface ExampleResult {
  body: DocumentNode;
  kind: ExampleResultKind;
}
export interface Member {
  docs: DocumentNode[];
  id: string;
  name: string;
  ty: ResolvedText;
}
export interface PageMetadata {
  title: string;
  description: string;
  canonical?: string | null;
  image?: string | null;
  noindex: boolean;
  ogType: OpenGraphType;
  structured: StructuredData[];
  twitterCard: TwitterCard;
  url?: string | null;
}
export interface BreadcrumbEntry {
  name: string;
  position: number;
  url?: string | null;
}
export interface Overload {
  docs: DocumentNode[];
  flags: string[];
  id: string;
  label?: string | null;
  parameters: Parameter[];
  platforms: string[];
  results: Parameter[];
  signature: ResolvedText;
  source: Location;
}
export interface Parameter {
  default: ResolvedText;
  docs: DocumentNode[];
  name: string;
  ty: ResolvedText;
}
export interface FormattedDefinition {
  anchors: DefinitionAnchor[];
  code: ResolvedText;
  docs: DocumentNode[];
  id: string;
  kind: DeclarationKind;
  platforms: string[];
  scope: string[];
  source: Location;
  sourceUrl?: string | null;
}
export interface DefinitionAnchor {
  end: number;
  id: string;
  name: string;
  start: number;
}
export interface DocumentPage {
  title: string;
  id: string;
  kind: DocumentKind;
  metadata: PageMetadata;
  nodes: DocumentNode[];
  outline: Heading[];
  source?: Location | null;
}
export interface Heading {
  id: string;
  label: string;
  level: number;
}
export interface SearchData {
  entries: SearchEntry[];
  schemaVersion: number;
}
export interface SearchEntry {
  binding: boolean;
  category?: string | null;
  experimental: boolean;
  id: string;
  kind: SearchKind;
  module?: string | null;
  name: string;
  overloads: SearchSignature[];
  preview?: SearchPreview | null;
  qualifiedName: string;
  summary: string;
  target: Target;
}
export interface SearchSignature {
  parameters: string[];
  results: string[];
}
export interface SearchParameter {
  default: string;
  name: string;
  ty: string;
}
export interface NavigationData {
  entries: NavigationDeclaration[];
  moduleId: string;
  schemaVersion: number;
}
export interface NavigationDeclaration {
  binding: boolean;
  experimental: boolean;
  group: string;
  id: string;
  kind: DeclarationKind;
  name: string;
  private: boolean;
  route: string;
}
