use super::super::categories;
use super::super::model::{Item, Kind, Member, Module, Parameter};
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
    assert_eq!(rows[1].href, "/docs/api/Vendor/Nested/MAX/");
    let page = bundle
        .items
        .iter()
        .find(|p| p.path == "api/Vendor/Nested/MAX/index.html")
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
            "base",
            "categories",
            "css_url",
            "footer_version",
            "format",
            "groups",
            "guides",
            "guides_index",
            "header_links",
            "index",
            "logo",
            "name",
            "nav",
            "repo",
            "script_url",
            "version"
        ]
    );
    assert_eq!(shared["format"], 1);
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
        ["id", "items", "page"]
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
        "/docs/api/Vendor/Nested/"
    );
}
