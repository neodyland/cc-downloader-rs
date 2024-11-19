use htmd::HtmlToMarkdown;
use regex::Regex;
static MAX_DISTANCE: usize = 25;

pub fn detect_language(body: &str) -> bool {
    let mut is_lang = false;
    for x in 'ぁ'..'ん' {
        if body.contains(x) {
            is_lang = true;
        }
    }
    for x in 'ァ'..'ン' {
        if body.contains(x) {
            is_lang = true;
        }
    }
    is_lang
}

fn detect_jp_ratio(re_jp: &Regex, s: &str) -> bool {
    let mut total_len = 0;
    for cap in re_jp.captures_iter(s) {
        if let Some(cap) = cap.get(0) {
            total_len += cap.as_str().chars().count();
        }
    }
    (total_len as f32) / (s.chars().count() as f32) > 0.55
}

pub fn extract(converter: &HtmlToMarkdown, re_jp: &Regex, text: &str) -> anyhow::Result<String> {
    let text = converter.convert(text)?;
    if text.chars().count() < 20 {
        anyhow::bail!("Too short")
    }
    let mut texts = vec!["".to_string()];
    let mut last_end = 0;
    let mut current_text_index = 0;
    for cap in re_jp.captures_iter(&text) {
        if let Some(cap) = cap.get(0) {
            let start = cap.start();
            let end = cap.end();
            let matched_text = cap.as_str();
            if last_end > 0 {
                if start - last_end > MAX_DISTANCE {
                    texts.push("".to_string());
                    current_text_index += 1;
                } else {
                    let intermediate = text[last_end..start].trim();
                    texts[current_text_index].push_str(intermediate);
                }
            } else {
                let intermediate = text[0..start].trim();
                let intermediate: String = intermediate
                    .chars()
                    .rev()
                    .take(50)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                texts[current_text_index].push_str(&intermediate);
            }
            texts[current_text_index].push_str(matched_text);
            last_end = end;
        }
    }
    texts = texts
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let text = texts.join("\n");
    if !detect_jp_ratio(re_jp, &text) {
        anyhow::bail!("Too few Japanese")
    }
    if text.chars().count() < 50 {
        anyhow::bail!("Too short")
    }
    Ok(text)
}
