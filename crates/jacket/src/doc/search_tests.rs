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
    // Each module, then its items: moving, steady. Above the low three bits, one more than the
    // index of the category whose badge they carry.
    assert_eq!(flags, [17, 17, 17, 0, 1, 0]);
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

#[test]
fn generated_declarations_carry_their_own_flag() {
    let made = |name: &str, generated| Item {
        name: name.into(),
        kind: Kind::Constant,
        generated,
        ..Item::default()
    };
    let site = Site::new(
        vec![Module {
            id: "Gen".into(),
            items: vec![made("hand_written", false), made("made", true)],
            ..Module::default()
        }],
        Vec::new(),
        "/".into(),
        "0.0.0".into(),
        String::new(),
        String::new(),
        false,
    );
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&site)).unwrap();
    let flags: Vec<i64> = index["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[6].as_i64().unwrap())
        .collect();
    // The module, then hand_written, made.
    assert_eq!(flags, [0, 0, 4]);
}

fn files_site(item: Item) -> Site {
    Site::new(
        vec![Module {
            id: "Files".into(),
            items: vec![item],
            ..Module::default()
        }],
        Vec::new(),
        "/".into(),
        "0.0.0".into(),
        String::new(),
        String::new(),
        false,
    )
}

#[test]
fn every_definition_of_a_name_is_searchable() {
    let def = |value: &str| Item {
        name: "Handle".into(),
        kind: Kind::Constant,
        value: value.into(),
        ..Item::default()
    };
    let mut handle = def("s32");
    handle.variants = vec![def("*void")];
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&files_site(handle))).unwrap();
    let line = index["items"][1][4].as_str().unwrap();
    assert!(line.contains("s32") && line.contains("*void"), "{line}");
}

#[test]
fn a_procedure_is_found_by_the_types_of_every_definition() {
    use super::super::model::{Overload, Param};
    let def = |ty: &str| Item {
        name: "open".into(),
        kind: Kind::Procedure,
        overloads: vec![Overload {
            params: vec![Param {
                ty: ty.into(),
                ..Param::default()
            }],
            ..Overload::default()
        }],
        ..Item::default()
    };
    let mut open = def("int");
    open.variants = vec![def("string")];
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&files_site(open))).unwrap();
    let shapes = index["items"][1][5].as_array().unwrap();
    assert_eq!(shapes.len(), 2, "{shapes:?}");
}

#[test]
fn a_result_shows_every_overload_signature() {
    use super::super::model::{Overload, Param};
    let overload = |ty: &str| Overload {
        params: vec![Param {
            name: "x".into(),
            ty: ty.into(),
            ..Param::default()
        }],
        ..Overload::default()
    };
    let write = Item {
        name: "write".into(),
        kind: Kind::Procedure,
        overloads: vec![overload("int"), overload("string")],
        ..Item::default()
    };
    let index: serde_json::Value =
        serde_json::from_str(&super::super::search::index(&files_site(write))).unwrap();
    let line = index["items"][1][4].as_str().unwrap();
    assert!(line.contains("int") && line.contains("string"), "{line}");
}
