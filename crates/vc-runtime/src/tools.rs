//! Intelligent Runtime Tools: Web Search & Active Knowledge Learning.
//!
//! Provides real-time DuckDuckGo / Wikipedia web lookup (Method A)
//! and user teaching fact extraction & semantic memory ingestion (Method B).

use serde::{Deserialize, Serialize};

/// Result of an external web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub query: String,
    pub source: String,
    pub snippets: Vec<String>,
}

impl WebSearchResult {
    pub fn summary(&self) -> String {
        if self.snippets.is_empty() {
            "Không tìm thấy thông tin phù hợp trên web.".to_string()
        } else {
            self.snippets
                .iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

pub struct WebSearchTool;

impl WebSearchTool {
    /// Detect if the user input is seeking real-world information or asks if Aria knows something.
    pub fn should_search(input: &str) -> Option<String> {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();

        // 1. Explicit search prefixes
        let search_triggers = [
            "search ",
            "tra google ",
            "tra mạng ",
            "tìm kiếm ",
            "google ",
            "tìm trên mạng ",
        ];
        for trigger in &search_triggers {
            if lower.starts_with(trigger) {
                let q = trimmed[trigger.len()..].trim().to_string();
                if !q.is_empty() {
                    return Some(q);
                }
            }
        }

        // 2. Ignore anaphoric conversational queries that reference earlier chat turns
        let anaphoric_tokens = [
            "vừa nói",
            "vừa nhắc",
            "vừa bảo",
            "câu đó",
            "lời đó",
            "vừa nãy",
            "ở trên",
            "câu trước",
            "anh vừa nói",
            "em vừa nói",
        ];
        if anaphoric_tokens.iter().any(|t| lower.contains(t)) {
            return None;
        }

        // 3. Questions about entities, celebrities, songs, or whether Aria knows them
        let entity_triggers = [
            "có biết",
            "biết gì về",
            "là ai",
            "ai là",
            "thời tiết",
            "tin tức",
            "bài hát",
            "bài gì",
            "nghĩa là gì",
            "định nghĩa",
        ];
        let has_trigger = entity_triggers.iter().any(|t| lower.contains(t));

        if has_trigger {
            let mut cleaned = lower.clone();
            // Remove conversational filler tokens
            for prefix in &["em ", "bạn ", "aria ", "nè ", "ạ ", "ơi "] {
                cleaned = cleaned.replace(prefix, " ");
            }
            for phrase in &[
                "có biết",
                "k không",
                "không nè",
                "không ta",
                "không nhỉ",
                "không",
                "k nè",
                " k",
                "về",
                "ai là",
                "là ai",
                "thế nào",
                "hôm nay",
                "như thế nào",
                "giúp mình",
                "hỏi xíu",
            ] {
                cleaned = cleaned.replace(phrase, " ");
            }
            let core_q = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
            if core_q.len() >= 2 {
                // If query is specifically "mck", enrich to "rapper MCK" for best results
                if core_q.eq_ignore_ascii_case("mck") {
                    return Some("rapper MCK".into());
                }
                return Some(core_q);
            }
        }

        None
    }

    /// Perform a live web search using DuckDuckGo Lite with fallback to Vietnamese Wikipedia.
    pub fn search(query: &str) -> WebSearchResult {
        // Try DuckDuckGo Lite first
        if let Some(res) = Self::search_duckduckgo_lite(query) {
            if !res.snippets.is_empty() {
                return res;
            }
        }

        // Fallback to Vietnamese Wikipedia
        if let Some(res) = Self::search_wikipedia(query) {
            if !res.snippets.is_empty() {
                return res;
            }
        }

        WebSearchResult {
            query: query.to_string(),
            source: "none".to_string(),
            snippets: vec![],
        }
    }

    fn search_duckduckgo_lite(query: &str) -> Option<WebSearchResult> {
        let resp = ureq::post("https://lite.duckduckgo.com/lite/")
            .set(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&format!("q={}", urlencoding(query)))
            .ok()?;

        let body = resp.into_string().ok()?;
        let snippets = extract_ddg_snippets(&body);

        if snippets.is_empty() {
            None
        } else {
            Some(WebSearchResult {
                query: query.to_string(),
                source: "DuckDuckGo".to_string(),
                snippets,
            })
        }
    }

    fn search_wikipedia(query: &str) -> Option<WebSearchResult> {
        let url = format!(
            "https://vi.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&utf8=1",
            urlencoding(query)
        );
        let resp = ureq::get(&url)
            .set("User-Agent", "VirtualCharacterBot/1.0 (agent)")
            .call()
            .ok()?;

        let json: serde_json::Value = resp.into_json().ok()?;
        let items = json.get("query")?.get("search")?.as_array()?;

        let mut snippets = Vec::new();
        for item in items.iter().take(2) {
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
            let snippet = item.get("snippet").and_then(|s| s.as_str()).unwrap_or("");
            let clean_snippet = strip_html_tags(snippet);
            if !clean_snippet.is_empty() {
                snippets.push(format!("{}: {}", title, clean_snippet));
            }
        }

        if snippets.is_empty() {
            None
        } else {
            Some(WebSearchResult {
                query: query.to_string(),
                source: "Wikipedia".to_string(),
                snippets,
            })
        }
    }
}

/// A fact, preference, or identity teaching extracted from user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingFact {
    pub fact: String,
    pub is_user_preference: bool,
}

pub struct KnowledgeLearner;

impl KnowledgeLearner {
    /// Detect if the user is actively teaching Aria something or sharing personal facts/preferences.
    pub fn detect_teaching(input: &str) -> Option<TeachingFact> {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();

        // 1. Explicit teaching triggers:
        let explicit_markers = [
            "dạy cho em",
            "dạy em",
            "dạy bạn",
            "nhớ là",
            "sau này nhớ",
            "cho em biết là",
            "ghi nhớ nè",
            "nhớ nhé",
            "nhớ nhen",
            "em phải nhớ",
        ];
        for marker in &explicit_markers {
            if let Some(pos) = lower.find(marker) {
                let start = pos + marker.len();
                let mut remainder = trimmed[start..]
                    .trim_start_matches(|c| c == ':' || c == ',' || c == ' ')
                    .trim();
                for prefix in &["nè:", "nè", "nha:", "nha", "nhen:", "nhen", "nhé:", "nhé"] {
                    if let Some(stripped) = remainder.strip_prefix(prefix) {
                        remainder = stripped
                            .trim_start_matches(|c| c == ':' || c == ',' || c == ' ')
                            .trim();
                    }
                }
                if remainder.len() >= 4 {
                    return Some(TeachingFact {
                        fact: remainder.to_string(),
                        is_user_preference: false,
                    });
                }
            }
        }

        // 2. Personal preferences:
        let pref_markers = [
            "mình thích ",
            "tôi thích ",
            "anh thích ",
            "em thích ",
            "sở thích của mình là ",
            "mình rất thích ",
            "tôi rất thích ",
            "mình ghét ",
            "tôi ghét ",
        ];
        for marker in &pref_markers {
            if let Some(pos) = lower.find(marker) {
                let remainder = trimmed[pos..].trim().to_string();
                if remainder.len() >= 5 {
                    return Some(TeachingFact {
                        fact: remainder,
                        is_user_preference: true,
                    });
                }
            }
        }

        // 3. Declarative statements with "là" (X là Y)
        // e.g. "MCK là rapper Việt Nam", "Long là tên thật của MCK"
        if (lower.contains(" là ") || lower.contains(" la "))
            && !lower.contains('?')
            && !lower.ends_with(" hả")
            && !lower.ends_with(" à")
            && !lower.ends_with(" phải không")
            && !lower.ends_with(" k")
            && !lower.ends_with(" ko")
            && !lower.contains("ai là")
            && !lower.contains("là ai")
            && !lower.contains("gì là")
            && !lower.contains("là gì")
            && !lower.contains("ở đâu")
            && !lower.contains("khi nào")
            && !lower.contains("mấy giờ")
            && !lower.contains("bao nhiêu")
            && !lower.contains("sao lại")
            && !lower.contains("có phải")
            && !lower.contains("thế nào")
            && !lower.contains("chưa")
            && !lower.ends_with(" nhỉ")
            && !lower.ends_with(" ta")
            && trimmed.len() >= 8
        {
            return Some(TeachingFact {
                fact: trimmed.to_string(),
                is_user_preference: false,
            });
        }

        None
    }
}

// Helpers
fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn strip_html_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_ddg_snippets(html: &str) -> Vec<String> {
    let tag = "<td class='result-snippet'>";
    let tag_alt = "<td class=\"result-snippet\">";
    let mut snippets = Vec::new();

    let mut remaining = html;
    while let Some(pos) = remaining.find(tag).or_else(|| remaining.find(tag_alt)) {
        let offset = if remaining[pos..].starts_with(tag) {
            tag.len()
        } else {
            tag_alt.len()
        };
        let start = pos + offset;
        remaining = &remaining[start..];
        if let Some(end) = remaining.find("</td>") {
            let snippet_raw = &remaining[..end];
            let clean = strip_html_tags(snippet_raw);
            if !clean.is_empty() {
                snippets.push(clean);
                if snippets.len() >= 2 {
                    break;
                }
            }
            remaining = &remaining[end + 5..];
        } else {
            break;
        }
    }
    snippets
}
