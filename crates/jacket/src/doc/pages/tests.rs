use super::super::categories::Category;
use super::*;

fn site() -> Site {
    let item = |name: &str, experimental| Item {
        name: name.into(),
        kind: Kind::Constant,
        docs: "A value.".into(),
        experimental,
        ..Item::default()
    };
    let module = |id: &str, experimental| Module {
        id: id.into(),
        docs: "A module.".into(),
        category: None,
        group: "General".into(),
        experimental,
        items: vec![item("moving", true), item("steady", false)],
        ..Module::default()
    };
    let mut site = Site::new(
        vec![
            Module {
                category: Some(1),
                ..module("Extensions/Moving", true)
            },
            Module {
                category: Some(0),
                ..module("Steady", false)
            },
        ],
        vec!["General".into()],
        "/".into(),
        "0.0.0".into(),
        String::new(),
        String::new(),
        false,
    );
    site.categories = vec![
        Category {
            name: "Library".into(),
            ..Category::default()
        },
        Category {
            name: "Extensions".into(),
            badge: "extension".into(),
            tip: "Not part of the library".into(),
            color: "#1f7a8c".into(),
            color_dark: "#6cc4d4".into(),
        },
    ];
    site
}

const BADGE: &str = "badge exp";

#[test]
fn the_experimental_badge_is_beside_what_is_tagged() {
    let site = site();
    let moving = module_page(&site, 0);
    assert!(moving.contains("<span class=\"badge exp\""));
    assert!(!module_page(&site, 1).contains("<span class=\"badge exp\""));
    let page = item_page(&site, 0, &site.modules[0].items[0]);
    assert!(page.contains("<span class=\"badge exp\""));
    // An item in an experimental module is experimental too, tagged or not.
    assert!(item_page(&site, 0, &site.modules[0].items[1]).contains(BADGE));
    assert!(!item_page(&site, 1, &site.modules[1].items[1]).contains(BADGE));
}

#[test]
fn a_category_badge_marks_its_modules_and_their_items() {
    let site = site();
    let ext = "<span class=\"badge cat-1\" title=\"Not part of the library\">extension</span>";
    assert!(module_page(&site, 0).contains(ext));
    assert!(item_page(&site, 0, &site.modules[0].items[1]).contains(ext));
    assert!(!module_page(&site, 1).contains(ext));
    assert!(index_page(&site).contains("Moving<span class=\"badge exp sm\""));
    assert!(index_page(&site).contains("<span class=\"badge cat-1 sm\""));
    // The colours come with the page, and headings name the categories.
    let home = index_page(&site);
    assert!(home.contains(".cat-1{--c:#1f7a8c}"), "{home}");
    assert!(home.contains("<h2 class=\"section\">Extensions</h2>"));
}

#[test]
fn lists_mark_experimental_modules_and_items() {
    let site = site();
    let home = index_page(&site);
    assert!(home.contains("Moving<span class=\"badge exp sm\""));
    assert!(!home.contains("Steady<span"));
    // In a module's list, only the tagged item has a badge in its row; the sidebar marks the
    // experimental module with a dot.
    let list = module_page(&site, 1);
    assert_eq!(list.matches("class=\"badge exp sm\"").count(), 1);
    assert!(list.contains("Moving<span class=\"dot exp\""));
}

#[test]
fn search_results_carry_the_flag() {
    let site = site();
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&site)).unwrap();
    let flags: Vec<i64> = index["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[6].as_i64().unwrap())
        .collect();
    // Each module, then its items: moving, steady. Above the low two bits, one more than the
    // index of the category whose badge they carry.
    assert_eq!(flags, [9, 9, 9, 0, 1, 0]);
    assert_eq!(index["cats"][1][0], "extension");
}

fn binding_site() -> Site {
    let native = |name: &str, kind, value: &str| Item {
        name: name.into(),
        kind,
        value: value.into(),
        binding: true,
        ..Item::default()
    };
    let mode = Item {
        members: vec![super::super::model::Member {
            name: "FAST".into(),
            ty: "1".into(),
            ..Default::default()
        }],
        ..native("Mode", Kind::Enum, "")
    };
    let own = Item {
        name: "own".into(),
        kind: Kind::Constant,
        docs: "Ours.".into(),
        ..Item::default()
    };
    Site::new(
        vec![Module {
            id: "Vendor".into(),
            category: None,
            group: "General".into(),
            items: vec![own, native("MAX", Kind::Constant, "8"), mode],
            ..Module::default()
        }],
        vec!["General".into()],
        "/".into(),
        "0.0.0".into(),
        String::new(),
        String::new(),
        false,
    )
}

#[test]
fn foreign_declarations_follow_the_own_items_in_their_sections() {
    let site = binding_site();
    let page = module_page(&site, 0);
    let own = page.find("id=\"own\"").unwrap();
    let max = page.find("id=\"MAX\"").unwrap();
    assert!(own < max);
    assert!(page[max..].contains("class=\"badge sm foreign\""));
    assert!(page.contains("FAST"));
    assert!(!page.contains("href=\"/api/Vendor/MAX/\""));
    assert_eq!(site.item_href("Vendor", "MAX"), "/api/Vendor/#MAX");
    assert_eq!(site.item_href("Vendor", "own"), "/api/Vendor/own/");
}

#[test]
fn native_declarations_are_searchable_but_flagged() {
    let site = binding_site();
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&site)).unwrap();
    let flags: Vec<i64> = index["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[6].as_i64().unwrap())
        .collect();
    // The module, then own, MAX, Mode.
    assert_eq!(flags, [0, 0, 2, 2]);
}

#[test]
fn module_parameters_have_a_section_before_the_items() {
    let mut site = site();
    site.modules[1].parameters = vec![super::super::model::Parameter {
        name: "STRICT".into(),
        ty: "bool".into(),
        default: "false".into(),
        doc: "Checks more.".into(),
        refs: Vec::new(),
    }];
    let page = module_page(&site, 1);
    let at = page.find("id=\"parameters\"").unwrap();
    assert!(at < page.find("id=\"constants\"").unwrap());
    assert!(page.contains(
        "<span class=\"pname\">STRICT</span><span class=\"colon\">: </span><span class=\"ty\">bool</span><span class=\"colon\"> = </span><span class=\"k\">false</span>"
    ));
    assert!(page.contains("Checks more."));
    assert!(page.contains("href=\"#parameters\""));
    assert!(!module_page(&site, 0).contains("Module parameters"));
}
