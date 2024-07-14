use htmd::{options::Options, HtmlToMarkdown};

pub fn get_converter() -> HtmlToMarkdown {
    let mut opt = Options::default();
    opt.preformatted_code = true;
    HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .options(opt)
        .build()
}

pub fn extract(cvt: &HtmlToMarkdown, s: &str) -> Option<String> {
    cvt.convert(s).ok()
}
