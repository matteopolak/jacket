use super::super::contract::{OpenGraphType, StructuredData, TwitterCard};
use super::*;
use crate::cli::{Cli, Command};
use clap::Parser;
use test_support::TempDir;

fn publication() -> Publication {
    Publication {
        origin: "https://example.com".into(),
        base: "/docs/".into(),
        name: "Owned docs".into(),
        image: "https://example.com/image.png".into(),
        version: "current".into(),
        canonical_version: None,
        image_file: None,
    }
}

fn args(flags: &[&str]) -> DocArgs {
    let cli =
        Cli::try_parse_from(["jacket", "doc"].into_iter().chain(flags.iter().copied())).unwrap();
    let Command::Doc(args) = cli.command else {
        panic!("expected doc")
    };
    args
}

fn site() -> Site {
    let mut site = Site::new(
        vec![],
        vec![],
        "/docs/".into(),
        "current".into(),
        String::new(),
        String::new(),
        false,
    );
    site.name = "Owned docs".into();
    site
}

#[test]
fn publication_preserves_root_archive_and_missing_origin_url_policy() {
    let mut p = publication();
    assert_eq!(
        p.url("index.html", false).as_deref(),
        Some("https://example.com/docs/")
    );
    assert_eq!(
        p.url("api/Example/Entry.html", false).as_deref(),
        Some("https://example.com/docs/api/Example/Entry")
    );
    p.canonical_version = Some("preferred".into());
    p.base = "/docs/versions/current/".into();
    assert_eq!(
        p.url("api/Example.html", true).as_deref(),
        Some("https://example.com/docs/versions/preferred/api/Example")
    );
    assert_eq!(
        p.url("api/Example.html", false).as_deref(),
        Some("https://example.com/docs/versions/current/api/Example")
    );
    p.origin.clear();
    assert!(p.url("api/Example.html", false).is_none());
    assert!(p.files(&[]).is_empty());
}

#[test]
fn typed_page_metadata_preserves_private_404_and_structured_data_policies() {
    let p = publication();
    let description = [DocumentNode::Paragraph {
        children: vec![DocumentNode::Text {
            text: "  Safe <literal> & text\n ".into(),
        }],
    }];
    let crumbs = [
        Breadcrumb {
            name: "Library".into(),
            href: Some("/docs/".into()),
        },
        Breadcrumb {
            name: "Example".into(),
            href: Some("/docs/api/Example".into()),
        },
        Breadcrumb {
            name: "Entry".into(),
            href: None,
        },
    ];
    let page = Page {
        path: "api/Example/Entry.html",
        kind: PageKind::Declaration,
        title: "Entry — Owned docs",
        description: &description,
        private: false,
        breadcrumbs: &crumbs,
    };
    let metadata = p.page(&page);
    assert_eq!(metadata.title, page.title);
    assert_eq!(metadata.description, "Safe <literal> & text");
    assert_eq!(metadata.og_type, OpenGraphType::Article);
    assert_eq!(metadata.twitter_card, TwitterCard::SummaryLargeImage);
    assert!(!metadata.noindex);
    assert_eq!(metadata.structured.len(), 2);
    assert!(
        matches!(&metadata.structured[0], StructuredData::TechArticle { module_name, .. } if module_name == "Example")
    );
    let StructuredData::BreadcrumbList {
        entries,
    } = &metadata.structured[1]
    else {
        panic!("breadcrumbs")
    };
    assert_eq!(
        entries.iter().map(|c| c.position).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(entries[0].url.as_deref(), Some("https://example.com/docs/"));
    assert_eq!(entries[2].url, metadata.url);
    let private = p.page(&Page {
        private: true,
        ..page
    });
    assert!(private.noindex);
    let missing = p.page(&Page {
        kind: PageKind::NotFound,
        path: "404.html",
        breadcrumbs: &[],
        ..page
    });
    assert!(missing.noindex);
    assert!(missing.url.is_none() && missing.canonical.is_none());
    assert!(missing.structured.is_empty());
    let home = p.page(&Page {
        kind: PageKind::Home,
        path: "",
        breadcrumbs: &[],
        ..page
    });
    assert_eq!(home.title, p.name);
    assert_eq!(home.og_type, OpenGraphType::Website);
    assert!(home.structured.is_empty());
    let module = p.page(&Page {
        kind: PageKind::Module,
        breadcrumbs: &crumbs[..2],
        ..page
    });
    assert!(matches!(
        module.structured.as_slice(),
        [StructuredData::BreadcrumbList { .. }]
    ));
}

#[test]
fn configuration_keeps_flag_precedence_and_local_image_provenance() {
    let temp = TempDir::new("astro-publication");
    temp.write("config/image.png", "config image");
    temp.write("working/image.png", "flag image");
    let doc = Doc {
        url: Some("https://config.example".into()),
        og_image: Some("image.png".into()),
        site_name: Some("Social name".into()),
        ..Doc::default()
    };
    let p = prepare(
        &args(&[]),
        &doc,
        &temp.join("config"),
        &temp.join("working"),
        &site(),
    )
    .unwrap();
    assert_eq!(p.name, "Social name");
    assert_eq!(p.image, "https://config.example/docs/assets/og-image.png");
    assert_eq!(p.image_file.as_ref().unwrap().1, b"config image");
    let p = prepare(
        &args(&[
            "--site-url",
            "https://flag.example/",
            "--og-image",
            "image.png",
        ]),
        &doc,
        &temp.join("config"),
        &temp.join("working"),
        &site(),
    )
    .unwrap();
    assert_eq!(p.origin, "https://flag.example");
    assert_eq!(p.image_file.as_ref().unwrap().1, b"flag image");
    let files = p.files(&[p.url("", false).unwrap()]);
    assert!(
        files
            .iter()
            .any(|asset| asset.path == "assets/og-image.png")
    );
    assert_eq!(
        files
            .iter()
            .find(|asset| asset.path == "assets/og-image.png")
            .unwrap()
            .provenance,
        super::super::contract::AssetProvenance::ConfiguredImage
    );
    assert!(
        files
            .iter()
            .filter(|asset| asset.path != "assets/og-image.png")
            .all(|asset| asset.provenance
                == super::super::contract::AssetProvenance::GeneratedMetadata)
    );
    assert!(files.iter().any(|asset| asset.path == "sitemap.xml"));
    assert!(files.iter().any(|asset| asset.path == "robots.txt"
        && asset.bytes == b"Sitemap: https://flag.example/docs/sitemap.xml\n"));
    for origin in [
        "https://",
        "https://example.com/path",
        "https://a@b",
        "javascript:alert(1)",
    ] {
        let doc = Doc {
            url: Some(origin.into()),
            ..Doc::default()
        };
        assert!(
            prepare(
                &args(&[]),
                &doc,
                &temp.join("config"),
                &temp.join("working"),
                &site()
            )
            .is_err()
        );
    }
    for version in ["", ".", "..", "nested/path", "query?value"] {
        let doc = Doc {
            canonical_version: Some(version.into()),
            ..Doc::default()
        };
        assert!(
            prepare(
                &args(&[]),
                &doc,
                &temp.join("config"),
                &temp.join("working"),
                &site()
            )
            .is_err()
        );
    }
}

#[test]
fn unknown_publication_origin_does_not_invent_breadcrumb_urls() {
    let mut publication = publication();
    publication.origin.clear();
    let crumbs = [
        Breadcrumb {
            name: "Library".into(),
            href: Some("/docs/".into()),
        },
        Breadcrumb {
            name: "Entry".into(),
            href: None,
        },
    ];
    let metadata = publication.page(&Page {
        path: "api/Example/Entry.html",
        kind: PageKind::Declaration,
        title: "Entry",
        description: &[],
        private: false,
        breadcrumbs: &crumbs,
    });
    let data = serde_json::to_value(&metadata).unwrap();
    let list = &data["structured"][1]["entries"];
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["url"].is_null()),
        "unknown origin must not produce relative or empty structured URLs: {list}"
    );
}
