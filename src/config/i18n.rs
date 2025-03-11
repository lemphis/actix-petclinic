use std::{collections::HashMap, fs, path::Path};

use actix_web::{http::header, HttpRequest};
use serde_json::Value;

const DEFAULT_LANGUAGE: &str = "en";
const ALLOWED_LANGUAGES: &[&str] = &[DEFAULT_LANGUAGE, "ko", "de", "es", "fa", "pt", "ru", "tr"];

#[derive(Clone)]
pub struct I18n {
    translations: HashMap<String, HashMap<String, Value>>,
}

impl I18n {
    pub fn new() -> Self {
        I18n {
            translations: Self::load_all_translations("locales/")
                .unwrap_or_else(|e| panic!("translation load error: {}", e)),
        }
    }

    fn load_all_translations<P: AsRef<Path>>(
        dir: P,
    ) -> Result<HashMap<String, HashMap<String, Value>>, Box<dyn std::error::Error>> {
        let mut all_translations = HashMap::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(lang) = path.file_stem().and_then(|s| s.to_str()) {
                    let translations = Self::load_translation_from_file(&path)?;
                    all_translations.insert(lang.to_string(), translations);
                }
            }
        }

        Ok(all_translations)
    }

    fn load_translation_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let translations = serde_json::from_str(&content)?;

        Ok(translations)
    }

    pub fn get(&self, req: &HttpRequest) -> &HashMap<String, Value> {
        let accept_language = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|h| h.to_str().ok());

        let preferred_language = Self::get_preferred_language(accept_language);

        // load_all_translations 연관 함수에 의해 translations 가 초기화되고,
        // preferred_language 는 적절한 language 를 찾지 못하더라도 "en" 을 return 하므로,
        // None 이 아님이 보장됨
        self.translations.get(&preferred_language).unwrap()
    }

    fn get_preferred_language(header: Option<&str>) -> String {
        let header = match header {
            Some(h) if !h.trim().is_empty() => h,
            _ => return DEFAULT_LANGUAGE.to_string(),
        };

        let languages = Self::parse_accept_language(header);
        for lang in languages {
            if ALLOWED_LANGUAGES.contains(&lang.as_str()) {
                return lang;
            }
        }

        DEFAULT_LANGUAGE.to_string()
    }

    fn parse_accept_language(header: &str) -> Vec<String> {
        let mut languages = Vec::new();

        for entry in header.split(',') {
            let parts: Vec<&str> = entry.split(';').map(|s| s.trim()).collect();
            let lang = parts[0].to_string();
            let q_value = if parts.len() > 1 && parts[1].starts_with("q=") {
                parts[1][2..].parse::<f32>().unwrap_or(1.0)
            } else {
                1.0
            };

            languages.push((lang.clone(), q_value));
        }

        languages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        languages.into_iter().map(|(lang, _)| lang).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_header() {
        let none_header = None;
        assert_eq!(
            I18n::get_preferred_language(none_header),
            DEFAULT_LANGUAGE.to_string()
        );
    }

    #[test]
    fn test_empty_header() {
        let empty_header = Some("");
        assert_eq!(
            I18n::get_preferred_language(empty_header),
            DEFAULT_LANGUAGE.to_string()
        );

        let space_header = Some("   ");
        assert_eq!(
            I18n::get_preferred_language(space_header),
            DEFAULT_LANGUAGE.to_string()
        );
    }

    #[test]
    fn test_unsupported_language() {
        let unsupported_language = Some("fr");
        assert_eq!(
            I18n::get_preferred_language(unsupported_language),
            DEFAULT_LANGUAGE.to_string()
        );
    }

    #[test]
    fn test_multiple_languages() {
        let accept_language_header = Some("ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7");
        assert_eq!(I18n::get_preferred_language(accept_language_header), "ko");
    }

    #[test]
    fn test_parse_accept_language() {
        let accept_language_header = "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7";
        assert_eq!(
            I18n::parse_accept_language(accept_language_header),
            ["ko-KR", "ko", "en-US", DEFAULT_LANGUAGE]
        );
    }
}
