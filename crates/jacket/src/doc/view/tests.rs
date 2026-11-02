use super::super::categories;
use super::super::model::{Item, Kind, Member, Module, Parameter, TypeRef};
use super::super::site::Site;
use super::*;

fn fixture() -> Site {
    let mut site = Site::new(
        vec![
            Module {
                id: "Vendor/Nested".into(),
                docs: "A module.\n\n```jai\nx := 1;\n```".into(),
                experimental: true,
                category: Some(0),
                parameters: vec![Parameter {
                    name: "STRICT".into(),
                    ty: "bool".into(),
                    default: "false".into(),
                    doc: "Checks more.".into(),
                    refs: Vec::new(),
                }],
                // Deliberately put a foreign item first in source order.
                items: vec![
                    Item {
                        name: "MAX".into(),
                        kind: Kind::Constant,
                        binding: true,
                        value: "8".into(),
                        ..Item::default()
                    },
                    Item {
                        name: "own".into(),
                        kind: Kind::Constant,
                        docs: "Ours.".into(),
                        ..Item::default()
                    },
                    Item {
                        name: "Mode".into(),
                        kind: Kind::Enum,
                        binding: true,
                        members: vec![Member {
                            name: "FAST".into(),
                            ty: "1".into(),
                            ..Member::default()
                        }],
                        ..Item::default()
                    },
                ],
                ..Module::default()
            },
            Module {
                id: "Steady".into(),
                items: vec![Item {
                    name: "still".into(),
                    kind: Kind::Constant,
                    ..Item::default()
                }],
                ..Module::default()
            },
        ],
        Vec::new(),
        "/docs/".into(),
        "0.31.0".into(),
        String::new(),
        String::new(),
        false,
    );
    site.categories = vec![categories::Category {
        name: "Extensions".into(),
        badge: "extension".into(),
        tip: "Not part of the library".into(),
        color: "#1f7a8c".into(),
        color_dark: "#6cc4d4".into(),
    }];
    site
}

#[test]
fn badges_and_flags_are_resolved_before_rendering() {
    let site = fixture();
    let module = module(&site, 0);
    let Content::Module {
        title,
        sections,
        ..
    } = &module.page.content
    else {
        panic!()
    };
    assert!(title.experimental);
    let badge = title.category.as_ref().unwrap();
    assert_eq!(
        (&*badge.class, &*badge.label, &*badge.tip),
        ("cat-0", "extension", "Not part of the library")
    );
    assert!(
        sections
            .iter()
            .flat_map(|s| &s.groups)
            .flat_map(|g| &g.rows)
            .all(|r| r.experimental)
    );
    for page in &module.items {
        let Content::Item {
            title, ..
        } = &page.content
        else {
            panic!()
        };
        assert!(title.experimental);
        assert_eq!(title.category.as_ref().unwrap().class, "cat-0");
    }
    let steady = super::module(&site, 1);
    let Content::Module {
        title, ..
    } = &steady.page.content
    else {
        panic!()
    };
    assert!(!title.experimental);
    assert!(title.category.is_none());
    let shared = super::site(&site);
    assert!(
        shared
            .nav
            .iter()
            .flat_map(|c| &c.groups)
            .flat_map(|g| &g.links)
            .any(|l| l.experimental && l.label == "Nested")
    );
    assert!(shared.categories[0].styled && shared.categories[0].dark_override);
    let search: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&site)).unwrap();
    let flags: Vec<_> = search["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[6].as_u64().unwrap())
        .collect();
    assert_eq!(flags, [5, 7, 5, 7, 0, 0]);
    assert_eq!(search["cats"][0][0], "extension");
}

#[test]
fn foreign_rows_follow_own_rows_and_have_item_pages() {
    let bundle = module(&fixture(), 0);
    let Content::Module {
        sections, ..
    } = &bundle.page.content
    else {
        panic!()
    };
    let constants = sections.iter().find(|s| s.id == "constants").unwrap();
    let rows = &constants.groups[0].rows;
    assert_eq!(
        rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        ["own", "MAX"]
    );
    assert!(!rows[0].foreign && rows[1].foreign);
    assert_eq!(rows[1].href, "/docs/api/Vendor/Nested/MAX");
    let page = bundle
        .items
        .iter()
        .find(|p| p.path == "api/Vendor/Nested/MAX.html")
        .unwrap();
    let Content::Item {
        title,
        value_html,
        ..
    } = &page.content
    else {
        panic!()
    };
    assert!(title.foreign);
    assert!(!value_html.is_empty());
    let mode = sections.iter().find(|s| s.id == "types").unwrap();
    assert_eq!(mode.groups[0].rows[0].members[0].name, "FAST");
}

#[test]
fn parameters_and_outline_are_data() {
    let bundle = module(&fixture(), 0);
    let Content::Module {
        parameters, ..
    } = &bundle.page.content
    else {
        panic!()
    };
    let p = &parameters[0];
    assert_eq!(p.name, "STRICT");
    assert_eq!(p.type_html, "<span class=\"ty\">bool</span>");
    assert_eq!(p.default_html, "<span class=\"k\">false</span>");
    assert_eq!(p.default_separator, " = ");
    assert_eq!(p.doc_html, "Checks more.");
    assert!(
        matches!(&bundle.page.outline[0],Outline::Heading { id,label,sub: false } if id == "parameters" && label == "Module parameters")
    );
    let steady = module(&fixture(), 1);
    let Content::Module {
        parameters, ..
    } = &steady.page.content
    else {
        panic!()
    };
    assert!(parameters.is_empty());
}

#[test]
fn json_contract_has_tagged_content_and_ordered_segments() {
    let site = fixture();
    let shared = serde_json::to_value(super::site(&site)).unwrap();
    let keys: Vec<_> = shared
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "assets",
            "base",
            "categories",
            "footer_version",
            "format",
            "groups",
            "guide_files",
            "header_links",
            "index",
            "logo",
            "module_files",
            "name",
            "nav",
            "not_found",
            "repo",
            "resolve_links",
            "site_name",
            "version",
            "written_files"
        ]
    );
    assert_eq!(shared["format"], 5);
    assert_eq!(shared["index"]["content"]["kind"], "index");
    assert_eq!(shared["index"]["sidebar"]["kind"], "modules");
    let bundle = serde_json::to_value(module(&site, 0)).unwrap();
    assert_eq!(
        bundle
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["id", "items", "nav", "page"]
    );
    let content = &bundle["page"]["content"];
    assert_eq!(content["kind"], "module");
    assert_eq!(
        content["docs"][0],
        serde_json::json!({"kind":"html","prose_html":"<p class=\"lead\">A module.</p>\n"})
    );
    let example = &content["docs"][1];
    assert_eq!(example["kind"], "example");
    assert!(
        example["example"]["code_html"]
            .as_str()
            .unwrap()
            .contains('x')
    );
    assert!(example["example"]["id"].is_null());
    assert!(!example["example"]["show_note"].as_bool().unwrap());
    assert_eq!(bundle["items"][0]["content"]["kind"], "item");
    assert_eq!(bundle["items"][0]["sidebar"]["kind"], "item");
    assert_eq!(
        bundle["items"][0]["sidebar"]["back"]["href"],
        "/docs/api/Vendor/Nested"
    );
}

#[test]
fn grouped_procedures_list_each_item_once() {
    let procedure = |name: &str, group: &str, binding: bool| Item {
        name: name.into(),
        kind: Kind::Procedure,
        group: group.into(),
        binding,
        ..Item::default()
    };
    let mut site = fixture();
    site.modules[1].items = vec![
        procedure("open", "Window", false),
        procedure("play", "Audio", false),
        procedure("SDL_Init", "Window", true),
    ];
    let bundle = module(&site, 1);
    let Content::Module {
        sections, ..
    } = &bundle.page.content
    else {
        panic!()
    };
    let procedures = sections.iter().find(|s| s.id == "procedures").unwrap();
    let rows: Vec<_> = procedures
        .groups
        .iter()
        .flat_map(|g| g.rows.iter().map(|r| r.id.as_str()))
        .collect();
    assert_eq!(rows, ["open", "play", "SDL_Init"]);
}

fn declared_site() -> Site {
    let reference = |path: &str, file: &str, line: usize, private: bool| TypeRef {
        path: path.into(),
        file: file.into(),
        line,
        private,
    };
    Site::new(
        vec![
            Module {
                id: "Io".into(),
                items: vec![Item {
                    name: "Channel".into(),
                    kind: Kind::Struct,
                    file: "Io/channel.jai".into(),
                    line: 5,
                    scope: vec!["T".into()],
                    declaration: "Channel :: struct(T: Type) {\n    senders: Waiter;\n    allocator: Allocator;\n    next: *Channel(T);\n}".into(),
                    decl_refs: vec![
                        reference("Waiter", "Io/sync.jai", 3, true),
                        reference("Allocator", "prelude/allocation.jai", 31, true),
                        reference("Channel", "Io/channel.jai", 5, false),
                    ],
                    members: vec![
                        Member {
                            name: "senders".into(),
                            ty: "Waiter".into(),
                            refs: vec![reference("Waiter", "Io/sync.jai", 3, true)],
                            ..Member::default()
                        },
                        Member {
                            name: "allocator".into(),
                            ty: "Allocator".into(),
                            refs: vec![reference("Allocator", "prelude/allocation.jai", 31, true)],
                            doc: "Where it allocates.".into(),
                        },
                    ],
                    ..Item::default()
                }],
                private: vec![Item {
                    name: "Waiter".into(),
                    kind: Kind::Struct,
                    file: "Io/sync.jai".into(),
                    line: 3,
                    declaration: "Waiter :: struct {\n    ticket: int;\n}".into(),
                    ..Item::default()
                }],
                ..Module::default()
            },
            Module {
                id: "prelude".into(),
                implicit: true,
                items: vec![Item {
                    name: "Allocator".into(),
                    kind: Kind::Struct,
                    file: "prelude/allocation.jai".into(),
                    line: 31,
                    declaration: "Allocator :: struct {\n    data: *void;\n}".into(),
                    ..Item::default()
                }],
                ..Module::default()
            },
        ],
        Vec::new(),
        "/docs/".into(),
        "0.32.0".into(),
        String::new(),
        String::new(),
        false,
    )
}

#[test]
fn a_declaration_links_private_and_prelude_types_and_not_itself() {
    let site = declared_site();
    let bundle = module(&site, 0);
    let Content::Module {
        sections, ..
    } = &bundle.page.content
    else {
        panic!()
    };
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].groups[0].rows.len(), 1);
    let row = &sections[0].groups[0].rows[0];
    assert!(row.lines_html.is_empty() && row.members.is_empty());
    let declaration = &row.declaration_html;
    assert!(
        declaration.contains("<a href=\"/docs/api/Io/Waiter\">Waiter</a>"),
        "{declaration}"
    );
    assert!(
        declaration.contains("<a href=\"/docs/api/prelude/Allocator\">Allocator</a>"),
        "{declaration}"
    );
    assert!(!declaration.contains("api/Io/Channel"), "{declaration}");
    assert!(declaration.contains("<span class=\"k\">struct</span>"));
    // Only a field with something to say keeps a row of the table.
    let page = bundle
        .items
        .iter()
        .find(|p| p.path == "api/Io/Channel.html")
        .unwrap();
    let Content::Item {
        members,
        declaration_html,
        private,
        ..
    } = &page.content
    else {
        panic!()
    };
    assert!(!private && !declaration_html.is_empty());
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "allocator");
}

#[test]
fn a_private_type_has_a_page_but_is_in_no_list() {
    let site = declared_site();
    let bundle = module(&site, 0);
    assert_eq!(bundle.items.len(), 2);
    let page = &bundle.items[1];
    assert_eq!(page.path, "api/Io/Waiter.html");
    let Content::Item {
        private,
        title,
        declaration_html,
        ..
    } = &page.content
    else {
        panic!()
    };
    assert!(*private && title.label == "Waiter" && declaration_html.contains("ticket"));
    // Its sidebar has no neighbours to list, and the shared lists hold only the exported type.
    let Sidebar::Item {
        nav,
        start,
        end,
        heading,
        ..
    } = &page.sidebar
    else {
        panic!()
    };
    assert!(heading.is_empty() && start == end);
    assert!(bundle.nav[*nav].is_empty());
    let listed: Vec<_> = bundle
        .nav
        .iter()
        .flatten()
        .map(|l| l.label.as_str())
        .collect();
    assert_eq!(listed, ["Channel"]);
    let outline = serde_json::to_string(&bundle.page.outline).unwrap();
    assert!(!outline.contains("Waiter"));
    let search = super::super::search::index(&site);
    assert!(!search.contains("Waiter"));
    assert!(
        site.unlinked_types().is_empty(),
        "{:?}",
        site.unlinked_types()
    );
    assert_eq!(site.item_href("Io", "Waiter"), "/docs/api/Io/Waiter");
}

#[test]
fn the_prelude_page_has_nothing_to_import() {
    let site = declared_site();
    let bundle = module(&site, 1);
    let Content::Module {
        import,
        implicit,
        ..
    } = &bundle.page.content
    else {
        panic!()
    };
    assert!(*implicit && import.is_empty());
    let io = module(&site, 0);
    let Content::Module {
        import, ..
    } = &io.page.content
    else {
        panic!()
    };
    assert_eq!(import, "Io");
}
