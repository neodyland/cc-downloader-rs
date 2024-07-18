use crate::{cc_stream::detect_language, ft::LanguagePredictor};
use lindera::{DictionaryConfig, DictionaryKind, Mode, Tokenizer, TokenizerConfig};
use once_cell::sync::Lazy;

static JP: Lazy<Tokenizer> = Lazy::new(|| {
    let dictionary = DictionaryConfig {
        kind: Some(DictionaryKind::UniDic),
        path: None,
    };

    let config = TokenizerConfig {
        dictionary,
        user_dictionary: None,
        mode: Mode::Normal,
    };
    Tokenizer::from_config(config).unwrap()
});

fn extract_sentence_filter(s: &String) -> Option<String> {
    let tok = JP.tokenize(s).ok()?;
    let mut tokens = 0;
    let mut not_noun_verb_adj = 0;
    for mut t in tok {
        tokens += 1;
        if let Some(d) = t.get_details() {
            let ty = d.first().unwrap_or(&"Unknown");
            match *ty {
                "名詞" | "動詞" | "形容詞" => {}
                _ => not_noun_verb_adj += 1,
            }
        };
    }
    let not_noun_verb_adj = not_noun_verb_adj as f32 / tokens as f32;
    if !(0.15..=0.85).contains(&not_noun_verb_adj) {
        return None;
    }
    let eng_ratio = s
        .char_indices()
        .filter(|(_, x)| x.is_ascii_alphanumeric())
        .count() as f32
        / s.char_indices().count() as f32;
    if eng_ratio > 0.4 {
        return None;
    }
    Some(s.to_string())
}

static BADWORDS: [&str; 92] = [
    "貧乳",
    "ヤリマン",
    "πモミモミ",
    "風俗",
    "関連項目",
    "sex",
    "SEX",
    "ナンパ",
    "死ね",
    "格安",
    "会員",
    "脱毛",
    "資格",
    "検定",
    "マンション",
    "ローン",
    "納品",
    "巨乳",
    "美乳",
    "オッパイ",
    "リフォーム",
    "リノベーション",
    "名無し",
    "セックス",
    "不動産",
    "リフォーム",
    "マンション",
    "保険",
    "引っ越し",
    "引越し",
    "商品",
    "マッサージ",
    "無料",
    "相続",
    "婚活",
    "フェラ",
    "ふぇら",
    "ピル",
    "ニキビ",
    "法律",
    "薬局",
    "歯医者",
    "健康",
    "症例",
    "避妊",
    "取扱い",
    "激安",
    "小遣い",
    "広告",
    "アーカイブ",
    "関連記事",
    "スポンサーリンク",
    "受付可能",
    "紹介しています",
    "いかがでしたか",
    "アフィリ",
    "稼いで",
    "抵抗がある",
    "お金がない",
    "稼げ",
    "情報商材",
    "副業",
    "アホ",
    "右翼",
    "左翼",
    "犯罪者",
    "上から目線",
    "在日",
    "送料無料",
    "美容液",
    "今だけ",
    "トップページ",
    "を探す",
    "求人一覧",
    "買取",
    "Posted by",
    "お問い合わせ",
    "[...]",
    "簡単な手順",
    "出会い",
    "すべて表示",
    "口コミ",
    "出逢い系アプリ",
    "出合いけいアプリ",
    "爆乳",
    "美肌",
    "レビュー",
    "キャンペーン",
    "ポイント",
    "分割払い",
    "リボ払い",
    "メルマガ",
];

pub fn extract(ft: &LanguagePredictor, text: &str) -> Vec<String> {
    let text = text.split('\n');
    let mut res = vec![];
    let mut tmp = String::new();
    let mut space_lines = 0;
    let mut joint_last = false;
    'tl: for t in text {
        let t = t.trim();
        if t.ends_with('。') && t.char_indices().count() > 10 && detect_language(ft, t) {
            if space_lines > 0 && space_lines < 3 {
                joint_last = true;
            }
            space_lines = 0;
            for bw in BADWORDS {
                if t.contains(bw) {
                    continue 'tl;
                }
            }
            tmp.push('\n');
            tmp.push_str(t);
        } else {
            space_lines += 1;
            let tmp_t = tmp.trim().to_string();
            if tmp_t.char_indices().count() > 50 {
                if joint_last && !res.is_empty() {
                    let index = res.len() - 1;
                    res[index] = format!("{}\n{}", res[res.len() - 1], tmp_t)
                } else {
                    res.push(tmp_t);
                }
            }
            joint_last = false;
            tmp.clear();
        }
    }
    res.iter().filter_map(extract_sentence_filter).collect()
}
