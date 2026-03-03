use crate::utils;
use html5gum::{EndTag, SpanBound, StartTag, Token, Tokenizer};
use serde_json::Value;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LogoPriority {
    JsonLd = 0,
    ManifestIcon = 1,
    AppleTouchIcon = 2,
    MsTile = 3,
    ImageSrc = 4,
    LargeIcon = 5,
    SquareOgImage = 6,
    SchemaImage = 7,
}

struct Parser<'a> {
    base_url: &'a str,
    logo: Option<(String, LogoPriority)>,
    manifest_url: Option<String>,
    in_json_ld: bool,
}

impl<'a> Parser<'a> {
    fn new(base_url: &'a str) -> Self {
        Self {
            base_url,
            logo: None,
            manifest_url: None,
            in_json_ld: false,
        }
    }

    fn set_logo(&mut self, url: String, priority: LogoPriority) {
        if url.is_empty() {
            return;
        }

        if let Some((_, current)) = &self.logo {
            if priority >= *current {
                return;
            }
        }

        self.logo = Some((url, priority));
    }

    fn handle_start<S: SpanBound>(&mut self, tag: &StartTag<S>) {
        let name = String::from_utf8_lossy(&tag.name).to_lowercase();

        match name.as_str() {
            "script" => self.check_json_ld(tag),
            "link" => self.check_link(tag),
            "meta" => self.check_meta(tag),
            "img" => self.check_img(tag),
            _ => {}
        }
    }

    fn handle_end<S: SpanBound>(&mut self, tag: &EndTag<S>) {
        if String::from_utf8_lossy(&tag.name).eq_ignore_ascii_case("script") {
            self.in_json_ld = false;
        }
    }

    fn handle_text(&mut self, bytes: &[u8]) {
        if !self.in_json_ld {
            return;
        }

        if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
            self.extract_from_json(&json);
        }
    }

    fn extract_from_json(&mut self, val: &Value) {
        match val {
            Value::Object(obj) => {
                if let Some(logo) = obj.get("logo") {
                    let url = match logo {
                        Value::String(s) => Some(s.as_str()),
                        Value::Object(o) => o
                            .get("url")
                            .or_else(|| o.get("@id"))
                            .or_else(|| o.get("contentUrl"))
                            .and_then(|v| v.as_str()),
                        _ => None,
                    };

                    if let Some(u) = url.filter(|s| !s.is_empty()) {
                        let normalized = utils::normalize_url(self.base_url, u);
                        self.set_logo(normalized, LogoPriority::JsonLd);
                        return;
                    }
                }

                for v in obj.values() {
                    self.extract_from_json(v);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.extract_from_json(item);
                }
            }
            _ => {}
        }
    }

    fn check_json_ld<S: SpanBound>(&mut self, tag: &StartTag<S>) {
        for (key, val) in &tag.attributes {
            let k = String::from_utf8_lossy(key).to_lowercase();
            let v = String::from_utf8_lossy(val).to_lowercase();

            if k == "type" && v.contains("application/ld+json") {
                self.in_json_ld = true;
            }
        }
    }

    fn check_link<S: SpanBound>(&mut self, tag: &StartTag<S>) {
        let mut rel = String::new();
        let mut href = String::new();
        let mut sizes = String::new();

        for (key, val) in &tag.attributes {
            match String::from_utf8_lossy(key).to_lowercase().as_str() {
                "rel" => rel = String::from_utf8_lossy(val).to_lowercase(),
                "href" => href = String::from_utf8_lossy(val).to_string(),
                "sizes" => sizes = String::from_utf8_lossy(val).to_lowercase(),
                _ => {}
            }
        }

        if href.is_empty() {
            return;
        }

        let url = utils::normalize_url(self.base_url, &href);

        if rel == "manifest" {
            self.manifest_url = Some(url);
        } else if rel.contains("apple-touch-icon") {
            self.set_logo(url, LogoPriority::AppleTouchIcon);
        } else if rel == "image_src" {
            self.set_logo(url, LogoPriority::ImageSrc);
        } else if rel.contains("icon") || rel.contains("shortcut") || rel.contains("mask-icon") {
            let size = utils::parse_max_icon_size(&sizes);

            if size >= 128 {
                self.set_logo(url, LogoPriority::LargeIcon);
            }
        }
    }

    fn check_meta<S: SpanBound>(&mut self, tag: &StartTag<S>) {
        let mut name_or_prop = None;
        let mut content = None;

        for (key, val) in &tag.attributes {
            match String::from_utf8_lossy(key).to_lowercase().as_str() {
                "name" | "property" => name_or_prop = Some(String::from_utf8_lossy(val).to_lowercase()),
                "content" => content = Some(String::from_utf8_lossy(val).to_string()),
                _ => {}
            }
        }

        if let (Some(prop), Some(url)) = (name_or_prop, content) {
            match prop.as_str() {
                "msapplication-tileimage" => {
                    let normalized = utils::normalize_url(self.base_url, &url);
                    self.set_logo(normalized, LogoPriority::MsTile);
                }
                "og:image" | "twitter:image" => {
                    if self.maybe_logo(&url) {
                        let normalized = utils::normalize_url(self.base_url, &url);
                        self.set_logo(normalized, LogoPriority::SquareOgImage);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_img<S: SpanBound>(&mut self, tag: &StartTag<S>) {
        let mut itemprop = None;
        let mut src = None;

        for (key, val) in &tag.attributes {
            match String::from_utf8_lossy(key).to_lowercase().as_str() {
                "itemprop" => itemprop = Some(String::from_utf8_lossy(val).to_lowercase()),
                "src" => src = Some(String::from_utf8_lossy(val).to_string()),
                _ => {}
            }
        }

        if let (Some(prop), Some(url)) = (itemprop, src) {
            if prop == "logo" || prop == "image" {
                let normalized = utils::normalize_url(self.base_url, &url);
                let priority = if prop == "logo" {
                    LogoPriority::JsonLd
                } else {
                    LogoPriority::SchemaImage
                };
                self.set_logo(normalized, priority);
            }
        }
    }

    fn maybe_logo(&self, url: &str) -> bool {
        let lower = url.to_lowercase();

        let has_logo_hint = lower.contains("logo")
            || lower.contains("-icon")
            || lower.contains("/icon")
            || lower.contains("square");

        let is_banner = lower.contains("banner")
            || lower.contains("hero")
            || lower.contains("cover")
            || lower.contains("1200x630")
            || lower.contains("1200x627");

        has_logo_hint && !is_banner
    }
}

pub struct ExtractedData {
    pub logo_url: Option<String>,
    pub manifest_url: Option<String>,
    pub priority: Option<LogoPriority>,
}

pub fn extract_site_data(html: &str, base_url: &str) -> ExtractedData {
    let mut parser = Parser::new(base_url);

    for token in Tokenizer::new(html).flatten() {
        match token {
            Token::StartTag(tag) => parser.handle_start(&tag),
            Token::EndTag(tag) => parser.handle_end(&tag),
            Token::String(bytes) => parser.handle_text(&bytes),
            _ => {}
        }
    }

    ExtractedData {
        logo_url: parser.logo.as_ref().map(|(url, _)| url.clone()),
        manifest_url: parser.manifest_url,
        priority: parser.logo.map(|(_, p)| p),
    }
}