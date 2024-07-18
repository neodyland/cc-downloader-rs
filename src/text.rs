use crate::{cc_stream::detect_language, ft::LanguagePredictor};

pub fn extract(ft: &LanguagePredictor, text: &str) -> Vec<String> {
    let text = text.split('\n');
    let mut res = vec![];
    let mut tmp = String::new();
    for t in text {
        let t = t.trim();
        if t.ends_with('。') && t.len() > 10 && detect_language(ft, t) {
            tmp.push('\n');
            tmp.push_str(t);
        } else {
            let tmp_t = tmp.trim().to_string();
            if !tmp_t.is_empty() {
                res.push(tmp_t);
            }
            tmp.clear();
        }
    }
    res
}
