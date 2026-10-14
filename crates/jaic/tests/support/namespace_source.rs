pub const SOURCE: &str =
    include_str!("../../../../tests/compiler-semantics/constructor-namespace/main.jai");

pub const FILES: &[(&str, &str)] = &[
    (
        "/lib/Mutation/module.jai",
        include_str!(
            "../../../../tests/compiler-semantics/constructor-namespace/modules/Mutation/module.jai"
        ),
    ),
    (
        "/lib/Record/module.jai",
        include_str!(
            "../../../../tests/compiler-semantics/constructor-namespace/modules/Record/module.jai"
        ),
    ),
    (
        "/lib/Record/types.jai",
        include_str!(
            "../../../../tests/compiler-semantics/constructor-namespace/modules/Record/types.jai"
        ),
    ),
    (
        "/lib/Record/constructor.jai",
        include_str!(
            "../../../../tests/compiler-semantics/constructor-namespace/modules/Record/constructor.jai"
        ),
    ),
    (
        "/lib/Factory/module.jai",
        include_str!(
            "../../../../tests/compiler-semantics/constructor-namespace/modules/Factory/module.jai"
        ),
    ),
];
