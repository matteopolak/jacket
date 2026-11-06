//! Internal guide ordering facts shared while the old view owners are retired.
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct NavCategory {
    pub label: String,
    pub groups: Vec<NavGroup>,
}

#[derive(Serialize, Clone)]
pub struct NavGroup {
    pub label: String,
    pub key: String,
    pub open: bool,
    pub links: Vec<NavLink>,
}

#[derive(Serialize, Clone)]
pub struct NavLink {
    pub href: String,
    pub label: String,
    pub title: Option<String>,
    pub depth: usize,
    pub current: bool,
    pub experimental: bool,
}
