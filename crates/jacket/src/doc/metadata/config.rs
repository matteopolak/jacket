//! Publication settings and auxiliary page inputs, kept separate from model extraction.
use super::{Metadata, valid_origin};
use crate::cli::DocArgs;
use crate::doc::{
    site::Site,
    view::{self, SiteView},
};
use jacket_config::schema::Doc;
use std::path::Path;

pub struct Publication {
    pub metadata: Metadata,
    pub urls: Vec<String>,
    pub image_file: Option<(String, Vec<u8>)>,
}

pub fn prepare(
    args: &DocArgs,
    doc: &Doc,
    config_dir: &Path,
    cwd: &Path,
    site: &Site,
    shared: &mut SiteView,
) -> Result<Publication, String> {
    let pick = |flag: &Option<String>, setting: &Option<String>| {
        flag.clone().or_else(|| setting.clone()).unwrap_or_default()
    };
    let origin = pick(&args.site_url, &doc.url)
        .trim_end_matches('/')
        .to_owned();
    if origin.is_empty() {
        eprintln!(
            "jacket: warning: no [doc] url or --site-url; canonical URLs and sitemaps are omitted"
        );
    } else if !valid_origin(&origin) {
        return Err("site URL must be an absolute HTTP(S) origin".into());
    }
    if let Some(version) = &doc.canonical_version
        && (version.is_empty()
            || !version
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            || matches!(version.as_str(), "." | ".."))
    {
        return Err("canonical_version must be a version path segment".into());
    }
    let image = pick(&args.og_image, &doc.og_image);
    let mut image_file = None;
    let image = if image.is_empty() || image.starts_with("https://") || image.starts_with("http://")
    {
        image
    } else {
        let root = if args.og_image.is_some() {
            cwd
        } else {
            config_dir
        };
        let path = root.join(&image);
        let name = path
            .file_name()
            .ok_or("og_image needs a file name")?
            .to_string_lossy();
        let dest = format!("assets/og-{name}");
        image_file = Some((
            dest.clone(),
            std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?,
        ));
        if origin.is_empty() {
            String::new()
        } else {
            format!("{origin}{}{dest}", site.base)
        }
    };
    let metadata = Metadata {
        origin,
        base: site.base.clone(),
        name: doc.site_name.clone().unwrap_or_else(|| site.name.clone()),
        image,
        version: site.version.clone(),
        canonical_version: doc.canonical_version.clone(),
    };
    let mut urls = Vec::new();
    shared.index.title.clone_from(&metadata.name);
    metadata.apply(&mut shared.index);
    urls.push(metadata.url("", false));
    if let Some(page) = &mut shared.guides_index {
        metadata.apply(page);
        urls.push(metadata.url(&page.path, false));
    }
    let mut not_found = view::site(site).index;
    not_found.not_found = true;
    not_found.path = "404.html".into();
    not_found.title = site.title("Page not found");
    not_found.description = "Search the docs or choose a module.".into();
    metadata.apply(&mut not_found);
    shared.not_found = Some(not_found);
    Ok(Publication {
        metadata,
        urls,
        image_file,
    })
}
