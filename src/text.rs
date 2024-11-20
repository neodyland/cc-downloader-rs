use std::cmp::Ordering;

use lindera::dictionary::{load_dictionary_from_kind, DictionaryKind};
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;
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

pub fn extract(re_jp: &Regex, tok: &Tokenizer, text: &str) -> anyhow::Result<Vec<String>> {
    if text.chars().count() < 20 {
        anyhow::bail!("Too short")
    }
    let mut texts = vec!["".to_string()];
    let mut last_end = 0;
    let mut current_text_index = 0;
    for cap in re_jp.captures_iter(text) {
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
    let text = clean(tok, text)?;
    Ok(text)
}

pub fn get_tokenizer() -> Tokenizer {
    let dictionary = load_dictionary_from_kind(DictionaryKind::UniDic).unwrap();
    let segmenter = Segmenter::new(
        Mode::Normal,
        dictionary,
        None, // Assuming no user dictionary is provided
    );
    let tokenizer = Tokenizer::new(segmenter);
    tokenizer
}

/// Calculate score and cleaned sentence for a given input
fn calculate_sentence_score(tok: &Tokenizer, sentence: &str) -> (f64, String) {
    let tokens = match tok.tokenize(sentence) {
        Ok(t) => t,
        Err(_) => return (0.0, String::new()),
    };

    let mut content_words = Vec::new();
    let mut function_words = Vec::new();
    let mut cleaned_sentence = Vec::new();
    let mut start = true;

    // Process each token
    for mut token in tokens {
        let pos = token.get_detail(0).unwrap_or("Unknown").to_string();
        let pos = pos.as_str();
        let pos_one = token.get_detail(1).unwrap_or("Unknown").to_string();
        let pos_one = pos_one.as_str();
        let surface = token.text.to_string();
        let surface = surface.as_str();

        // Skip certain words at the start
        if start
            && (["助詞", "副詞", "接続詞", "助動詞"].contains(&pos)
                || ["、", ","].contains(&surface))
        {
            continue;
        } else {
            start = false;
            cleaned_sentence.push(surface.to_string());
        }

        // Categorize words
        if ["名詞", "動詞", "形容詞", "副詞"].contains(&pos)
            && !pos_one.contains("非自立")
            && !pos_one.contains("助動詞")
            && !pos_one.contains("空白")
            && !["する", "ある", "いる", "なる"].contains(&surface)
        {
            content_words.push(surface.to_string());
        } else if ["助詞", "助動詞"].contains(&pos) || pos_one.contains("非自立") {
            function_words.push(surface.to_string());
        }
    }

    // Calculate score
    let mut score = 1.0;

    // 1. Content vs function word ratio
    let total_words = content_words.len() + function_words.len();
    if total_words > 0 {
        let content_ratio = content_words.len() as f64 / total_words as f64;
        score *= 0.6 + content_ratio;
    }
    let cleaned_sentence = cleaned_sentence.join("");
    // 2. Length-based scoring
    let length = cleaned_sentence.chars().count();
    score *= match length {
        0..=9 => 0.0,
        10..=19 => 1.0,
        20..=80 => 1.2,
        81..=200 => 1.0,
        _ => 0.7,
    };

    // 3. Content word quality
    if content_words.len() >= 2 {
        score *= 1.2;
    }

    // 4. Number presence bonus
    if Regex::new(r"\d").unwrap().is_match(&cleaned_sentence) {
        score *= 1.3;
    }

    (score, cleaned_sentence)
}

/// Split text into sentences considering whitespace
fn split_into_sentences(text: &str) -> Vec<String> {
    text.split('。')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                // Take the last part after whitespace split
                Some(
                    trimmed
                        .split_whitespace()
                        .last()
                        .unwrap_or(trimmed)
                        .to_string(),
                )
            }
        })
        .collect()
}

/// Main cleaning function that processes text and returns selected sentences
pub fn clean(tok: &Tokenizer, text: String) -> anyhow::Result<Vec<String>> {
    let sentences = split_into_sentences(&text);

    // Calculate scores for each sentence
    let mut scored_sentences: Vec<_> = sentences
        .iter()
        .map(|s| calculate_sentence_score(tok, s))
        .enumerate()
        .collect();

    // Normalize scores
    if let Some(max_score) = scored_sentences
        .iter()
        .map(|(_, (score, _))| *score)
        .max_by(|x, y| {
            if y - x > 0.0 {
                Ordering::Greater
            } else if y == x {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        })
    {
        if max_score > 0.0 {
            for (_, (score, _)) in &mut scored_sentences {
                *score /= max_score;
            }
        }
    }

    // Filter sentences with score >= 0.5 and merge adjacent ones
    let threshold = 0.5;
    let mut result = Vec::new();
    let mut current_group = Vec::new();
    let mut prev_pos = None;

    for (pos, (score, sentence)) in scored_sentences {
        if score >= threshold {
            match prev_pos {
                Some(p) if pos - p < 3 => {
                    current_group.push(sentence + "。");
                }
                _ => {
                    if !current_group.is_empty() {
                        result.push(current_group.join(""));
                        current_group.clear();
                    }
                    current_group.push(sentence + "。");
                }
            }
            prev_pos = Some(pos);
        }
    }

    // Add the last group if any
    if !current_group.is_empty() {
        result.push(current_group.join(""));
    }
    Ok(result
        .into_iter()
        .filter(|x| x.chars().count() > 30)
        .collect())
}
