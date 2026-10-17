//! The Quarterly Earnings Call. Free of egui.

pub mod data;
pub mod earnings_call;
pub mod phrases;

use std::path::Path;
use std::sync::OnceLock;

use minijinja::Environment;
use minijinja::value::Serde;
use serde::Serialize;

use earnings_call::EarningsCall;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Html,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
        }
    }
}

const CSS: &str = include_str!("../../templates/report.css");

fn env() -> &'static Environment<'static> {
    static ENV: OnceLock<Environment<'static>> = OnceLock::new();
    ENV.get_or_init(|| {
        let mut env = Environment::new();
        for (name, source) in [
            (
                "earnings_call.md.j2",
                include_str!("../../templates/earnings_call.md.j2"),
            ),
            (
                "earnings_call.html.j2",
                include_str!("../../templates/earnings_call.html.j2"),
            ),
        ] {
            env.add_template(name, source)
                .expect("embedded templates are valid");
        }
        // Text placed in a Markdown table cell: one line, pipes escaped.
        env.add_filter("cell", |value: String| {
            value
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .replace('|', "\\|")
        });
        env
    })
}

#[derive(Serialize)]
struct WithStyle<'a, T: Serialize> {
    #[serde(flatten)]
    report: &'a T,
    css: &'static str,
}

pub fn render(call: &EarningsCall, format: Format) -> Result<String, minijinja::Error> {
    let name = format!("earnings_call.{}.j2", format.extension());
    let mut out = env().get_template(&name)?.render(Serde(WithStyle {
        report: call,
        css: CSS,
    }))?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Suggested file name: `Earnings Call Q4 2026.md`.
pub fn file_name(call: &EarningsCall, format: Format) -> String {
    let quarter = call.title.trim_end_matches(" Earnings Call");
    format!("Earnings Call {quarter}.{}", format.extension())
}

pub fn write(path: &Path, contents: &str) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use earnings_call::{prepare, tests::sample};

    #[test]
    fn markdown() {
        insta::assert_snapshot!(render(&prepare(&sample()), Format::Markdown).unwrap());
    }

    #[test]
    fn html() {
        let html = render(&prepare(&sample()), Format::Html).unwrap();
        assert!(html.contains("<style>:root"), "stylesheet embedded");
        insta::assert_snapshot!(html.split("</style>").nth(1).unwrap());
    }

    #[test]
    fn html_escapes_user_text() {
        let mut d = sample();
        d.name = "<script>alert(1)</script>".into();
        let html = render(&prepare(&d), Format::Html).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn file_names() {
        let call = prepare(&sample());
        assert_eq!(
            file_name(&call, Format::Markdown),
            "Earnings Call Q4 2026.md"
        );
        assert_eq!(file_name(&call, Format::Html), "Earnings Call Q4 2026.html");
    }
}
