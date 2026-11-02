// Versioned rendering input. Keep in sync with ../view.rs.
#import "Extensions/Variant";

Site_View :: struct {
    format: u32;
    name: string;
    site_name: string;
    base: string;
    version: string;
    logo: *Logo;
    header_links: [] Link;
    repo: string;
    categories: [] Category;
    groups: [] string;
    nav: [] Nav_Category;
    guides: [] Guide_Link;
    index: Page_View;
    guides_index: *Page_View;
    not_found: *Page_View; @json(default)
    css_url: string; @json(default)
    script_url: string; @json(default)
    categories_url: string; @json(default)
    sidebar_url: string; @json(default)
    icons_url: string; @json(default)
    footer_version: string;
    module_files: [] string;
    guide_files: [] string;
    written_files: [] string;
    assets: [] string;
    resolve_links: bool;
}

Logo :: struct {
    url: string;
    mime: string;
}

Category :: struct {
    name: string;
    class: string;
    badge: string;
    tip: string;
    color: string;
    color_dark: string;
    styled: bool;
    dark_override: bool;
}

Link :: struct {
    href: string;
    label: string;
}

Guide_Link :: struct {
    href: string;
    label: string;
    summary: string;
}

Module_View :: struct {
    id: string;
    page: Page_View;
    items: [] Page_View;
    nav: [][] Nav_Link;
}

Page_Head :: struct {
    url: string;
    canonical: string;
    og_type: string;
    image: string;
    twitter_card: string;
    noindex: bool;
    json_ld: string;
}

Page_View :: struct {
    title: string;
    head: Page_Head;
    not_found: bool; @json(default)
    description: string;
    path: string;
    side_label: string;
    sidebar: Sidebar; @json(tag=kind,tags=modules,item,guides)
    outline: [] Outline; @json(tag=kind,tags=heading,items)
    content: Content; @json(tag=kind,tags=index,module,item,guides_index,guide)
}

Sidebar_Modules :: struct {
    categories: [] Nav_Category;
}

Sidebar_Item :: struct {
    back: Link;
    heading: string;
    nav: int;
    start: int;
    end: int;
    current: int;
    anchors: [] Link;
}

Sidebar_Guides :: struct {
    heading: string;
    links: [] Nav_Link;
}

Sidebar :: Variant(Sidebar_Modules, Sidebar_Item, Sidebar_Guides);

Nav_Category :: struct {
    label: string;
    groups: [] Nav_Group;
}

Nav_Group :: struct {
    label: string;
    key: string;
    open: bool;
    links: [] Nav_Link;
}

Nav_Link :: struct {
    href: string;
    label: string;
    title: *string;
    depth: int;
    current: bool;
    experimental: bool;
}

Content_Index :: struct {
    categories: [] Card_Category;
}

Content_Module :: struct {
    crumbs: [] Crumb;
    title: Title;
    available: string;
    since: string;
    import: string;
    docs: [] Doc_Segment; @json(tag=kind,tags=html,example,code)
    parameters: [] Definition;
    sections: [] Section;
}

Content_Item :: struct {
    crumbs: [] Crumb;
    title: Title;
    summary_html: string;
    docs: [] Doc_Segment; @json(tag=kind,tags=html,example,code)
    overloads: [] Overload_View;
    value_html: string;
    members_label: string;
    members: [] Definition;
    see_also: [] Link;
}

Content_GuidesIndex :: struct {
    title: string;
    cards: [] Guide_Link;
}

Content_Guide :: struct {
    segments: [] Doc_Segment; @json(tag=kind,tags=html,example,code)
}

Content :: Variant(Content_Index, Content_Module, Content_Item, Content_GuidesIndex, Content_Guide);

Card_Category :: struct {
    label: string;
    groups: [] Card_Group;
}

Card_Group :: struct {
    label: string;
    cards: [] Card;
}

Card :: struct {
    href: string;
    label: string;
    experimental: bool;
    category: *Badge;
    summary_html: string;
}

Badge :: struct {
    class: string;
    label: string;
    tip: string;
}

Crumb :: struct {
    label: string;
    href: *string;
    mono: bool;
}

Title :: struct {
    label: string;
    kind: string;
    procedure: bool;
    foreign: bool;
    category: *Badge;
    experimental: bool;
    overloads: *int;
    platforms: string;
    since: string;
    source_url: *string;
    mono: bool;
}

Definition :: struct {
    id: *string;
    name: string;
    type_html: string;
    default_html: string;
    default_separator: string;
    doc_html: string;
}

Section :: struct {
    id: string;
    label: string;
    groups: [] Row_Group;
}

Row_Group :: struct {
    heading: *Link;
    rows: [] Row;
}

Row :: struct {
    id: string;
    href: string;
    kind_badge: string;
    foreign: bool;
    experimental: bool;
    overloads: *int;
    platforms: string;
    since: string;
    source_url: *string;
    lines_html: [] string;
    summary_html: string;
    members: [] Definition;
    example: *Example;
}

Overload_View :: struct {
    id: string;
    label: *string;
    card: bool;
    signature_html: string;
    prose: [] Doc_Segment; @json(tag=kind,tags=html,example,code)
    parameters: [] Definition;
    returns: [] Definition;
    examples: [] Doc_Segment; @json(tag=kind,tags=html,example,code)
}

DocSegment_Html :: struct {
    prose_html: string;
}

DocSegment_Example :: struct {
    example: Example;
}

DocSegment_Code :: struct {
    code_html: string;
}

Doc_Segment :: Variant(DocSegment_Html, DocSegment_Example, DocSegment_Code);

Example :: struct {
    plain: bool;
    compact: bool;
    code_html: string;
    id: *string;
    note: *string;
    show_note: bool;
    playground_url: *string;
}

Outline_Heading :: struct {
    id: string;
    label: string;
    sub: bool;
}

Outline_Items :: struct {
    items: [] Outline_Item;
}

Outline :: Variant(Outline_Heading, Outline_Items);

Outline_Item :: struct {
    id: string;
    label: string;
}
