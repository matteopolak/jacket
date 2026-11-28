//! Help text from a spec, in sections, with optional colour.
use crate::spec::Spec;

/// One line of a section: a label and what it means.
#[derive(Clone, Debug)]
pub struct Row {
    pub label: String,
    /// Further lines are indented under the first.
    pub text: String,
}

/// The pieces of a help page, for a tool to add its own sections to before rendering.
#[derive(Clone, Debug)]
pub struct Help {
    pub name: String,
    pub version: String,
    pub about: String,
    pub usage: String,
    pub description: String,
    pub sections: Vec<(String, Vec<Row>)>,
    /// Paragraphs after the sections.
    pub footer: Vec<String>,
}

const BOLD: &str = "\x1b[1m";
const HEADING: &str = "\x1b[1;4m";
const RESET: &str = "\x1b[0m";

/// How wide a label may be before its text goes on the next line.
const LABEL_WIDTH: usize = 24;

impl Spec {
    /// The help page its options and doc comments make.
    ///
    /// ```
    /// use argdef::Args;
    ///
    /// #[derive(Args, Default)]
    /// #[args(name = "demo", usage = "demo [options]")]
    /// struct Demo {
    ///     /// print more
    ///     #[arg(name = "-verbose", section = "Options")]
    ///     verbose: bool,
    ///     /// the output file
    ///     #[arg(name = "-out", value = "file", hint = "file")]
    ///     out: Option<String>,
    /// }
    ///
    /// let text = Demo::spec().help().render(false);
    /// assert!(text.contains("Usage: demo [options]"));
    /// assert!(text.contains("  -out <file>"));
    /// ```
    pub fn help(&self) -> Help {
        let meta = &self.meta;
        let mut titles: Vec<&str> = meta.sections.to_vec();

        for opt in self.opts.iter().filter(|o| o.listed()) {
            if !titles.contains(&opt.section) {
                titles.push(opt.section);
            }
        }

        let sections = titles
            .into_iter()
            .map(|title| {
                let rows: Vec<Row> = self
                    .opts
                    .iter()
                    .filter(|o| o.listed() && o.section == title)
                    .map(|o| Row {
                        label: o.label(),
                        text: o.help.to_string(),
                    })
                    .collect();

                (
                    if title.is_empty() {
                        "Options"
                    } else {
                        title
                    }
                    .to_string(),
                    rows,
                )
            })
            .filter(|(_, rows)| !rows.is_empty())
            .collect();

        Help {
            name: meta.name.to_string(),
            version: meta.version.to_string(),
            about: meta.about.to_string(),
            usage: meta.usage.to_string(),
            description: meta.description.to_string(),
            sections,
            footer: Vec::new(),
        }
    }
}

impl Help {
    /// Add a section after the ones there.
    ///
    /// ```
    /// use argdef::{Help, Row};
    ///
    /// let help = Help {
    ///     name: "demo".into(),
    ///     version: String::new(),
    ///     about: String::new(),
    ///     usage: String::new(),
    ///     description: String::new(),
    ///     sections: vec![],
    ///     footer: vec![],
    /// };
    /// let row = Row {
    ///     label: "DEMO_HOME".into(),
    ///     text: "where it lives".into(),
    /// };
    /// assert!(
    ///     help.section("Environment", vec![row])
    ///         .render(false)
    ///         .contains("DEMO_HOME")
    /// );
    /// ```
    #[must_use]
    pub fn section(mut self, title: &str, rows: Vec<Row>) -> Self {
        self.sections.push((title.to_string(), rows));
        self
    }

    /// Add a section before the one called `before` (at the end if there is none).
    ///
    /// ```
    /// use argdef::{Help, Row};
    ///
    /// let help = Help {
    ///     name: "demo".into(),
    ///     version: String::new(),
    ///     about: String::new(),
    ///     usage: String::new(),
    ///     description: String::new(),
    ///     sections: vec![],
    ///     footer: vec![],
    /// };
    /// let row = Row {
    ///     label: "A".into(),
    ///     text: "b".into(),
    /// };
    /// let text = help
    ///     .section("Last", vec![row.clone()])
    ///     .section_before("Last", "First", vec![row])
    ///     .render(false);
    /// assert!(text.find("First:").unwrap() < text.find("Last:").unwrap());
    /// ```
    #[must_use]
    pub fn section_before(mut self, before: &str, title: &str, rows: Vec<Row>) -> Self {
        let at = self
            .sections
            .iter()
            .position(|(t, _)| t == before)
            .unwrap_or(self.sections.len());

        self.sections.insert(at, (title.to_string(), rows));
        self
    }

    /// Add a paragraph at the end.
    ///
    /// ```
    /// # use argdef::Help;
    /// # let help = Help { name: "demo".into(), version: String::new(), about: String::new(),
    /// #     usage: String::new(), description: String::new(), sections: vec![], footer: vec![] };
    /// assert!(help.footer("Exit status: 0 success").render(false).ends_with("success"));
    /// ```
    #[must_use]
    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer.push(text.into());
        self
    }

    /// The page as text: bold headings and labels when `color`.
    ///
    /// ```
    /// use argdef::Args;
    ///
    /// #[derive(Args, Default)]
    /// #[args(name = "demo", usage = "demo [options]")]
    /// struct Demo {
    ///     /// print more
    ///     #[arg(name = "-verbose", section = "Options")]
    ///     verbose: bool,
    ///     /// the output file
    ///     #[arg(name = "-out", value = "file", hint = "file")]
    ///     out: Option<String>,
    /// }
    ///
    /// let help = Demo::spec().help();
    /// assert!(!help.render(false).contains('\x1b'));
    /// assert!(help.render(true).contains("\x1b[1m-verbose\x1b[0m"));
    /// ```
    pub fn render(&self, color: bool) -> String {
        let paint = |style: &str, text: &str| {
            if color {
                format!("{style}{text}{RESET}")
            } else {
                text.to_string()
            }
        };
        let mut out = String::new();
        let mut title = self.name.clone();

        if !self.version.is_empty() {
            title.push(' ');
            title.push_str(&self.version);
        }

        if !self.about.is_empty() {
            title.push_str(": ");
            title.push_str(&self.about);
        }

        out.push_str(&paint(BOLD, &title));
        out.push_str("\n\n");

        if !self.usage.is_empty() {
            out.push_str(&paint(HEADING, "Usage:"));
            out.push(' ');
            out.push_str(&paint(BOLD, &self.usage));
            out.push_str("\n\n");
        }

        if !self.description.is_empty() {
            out.push_str(&self.description);
            out.push_str("\n\n");
        }

        for (title, rows) in &self.sections {
            out.push_str(&paint(HEADING, &format!("{title}:")));
            out.push('\n');

            for row in rows {
                let mut lines = row.text.lines();
                let first = lines.next().unwrap_or_default();
                let width = row.label.chars().count();
                let label = paint(BOLD, &row.label);

                if width <= LABEL_WIDTH {
                    out.push_str(&format!(
                        "  {label}{:pad$}{first}\n",
                        "",
                        pad = LABEL_WIDTH + 2 - width
                    ));
                } else {
                    out.push_str(&format!("  {label}\n{:28}{first}\n", ""));
                }

                for more in lines {
                    out.push_str(&format!("{:28}{more}\n", ""));
                }
            }

            out.push('\n');
        }

        for paragraph in &self.footer {
            out.push_str(paragraph);
            out.push_str("\n\n");
        }

        while out.ends_with('\n') {
            out.pop();
        }

        out
    }
}
