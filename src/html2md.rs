use htmd::{options::Options, HtmlToMarkdown};

pub fn get_converter() -> HtmlToMarkdown {
    HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .options(Options {
            preformatted_code: true,
            ..Default::default()
        })
        .build()
}

pub fn extract(cvt: &HtmlToMarkdown, s: &str) -> Option<String> {
    cvt.convert(s).ok()
}
