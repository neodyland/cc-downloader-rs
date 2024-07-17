use htmd::{options::Options, HtmlToMarkdown};
use once_cell::sync::Lazy;
use regex::Regex;

static RE_IMG: Lazy<Regex> =
    Lazy::new(|| Regex::new("\\[??!??\\[([^\\]]*)\\]\\([^)]*\\)").unwrap());

static RE_HTML: Lazy<Regex> = Lazy::new(|| Regex::new("<[^>]+>[^<]+<[^>]+>|<[^>]+>").unwrap());

pub fn get_converter() -> HtmlToMarkdown {
    HtmlToMarkdown::builder()
        .skip_tags(vec![
            "script", "style", "header", "footer", "section", "nav", "img", "video", "iframe",
        ])
        .options(Options {
            preformatted_code: true,
            ..Default::default()
        })
        .build()
}

fn filter_line(line: &&str) -> bool {
    let lt = line.trim();
    if lt == "*" || lt == "-" {
        return false;
    }
    true
}

pub fn extract(cvt: &HtmlToMarkdown, text: &str) -> Option<Vec<String>> {
    let mut text = cvt.convert(text).ok()?;
    for _ in 0..5 {
        text = RE_IMG.replace_all(&text, "$1").to_string();
    }
    text = RE_HTML.replace_all(&text, "").to_string();
    text = text
        .split('\n')
        .filter(filter_line)
        .collect::<Vec<_>>()
        .join("\n");
    Some(
        text.split("\n\n\n")
            .filter_map(|x| {
                if x.split('\n')
                    .filter(|x| (4..=50).contains(&x.char_indices().count()))
                    .collect::<Vec<_>>()
                    .len()
                    * 2
                    / 3
                    < x.split('\n').collect::<Vec<_>>().len()
                    && x.trim().char_indices().count() > 50
                {
                    Some(x.trim().to_string())
                } else {
                    None
                }
            })
            .collect(),
    )
}
