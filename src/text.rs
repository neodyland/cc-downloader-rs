use crate::{cc_stream::detect_language, ft::LanguagePredictor};

static BADWORDS: [&str; 85] = [
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
];

pub fn extract(ft: &LanguagePredictor, text: &str) -> Vec<String> {
    let text = text.split('\n');
    let mut res = vec![];
    let mut tmp = String::new();
    'tl: for t in text {
        let t = t.trim();
        if t.ends_with('。') && t.len() > 10 && detect_language(ft, t) {
            for bw in BADWORDS {
                if t.contains(bw) {
                    continue 'tl;
                }
            }
            tmp.push('\n');
            tmp.push_str(t);
        } else {
            let tmp_t = tmp.trim().to_string();
            if tmp_t.len() > 50 {
                res.push(tmp_t);
            }
            tmp.clear();
        }
    }
    res
}
