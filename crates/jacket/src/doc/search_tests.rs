use super::super::categories::Category;
use super::super::model::{Item, Kind, Module};
use super::super::site::Site;

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
