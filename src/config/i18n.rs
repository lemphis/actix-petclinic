use std::{collections::HashMap, fs, path::Path};

use actix_web::{http::header, HttpRequest};
use serde_json::Value;
use validator::ValidationErrors;

const DEFAULT_LANGUAGE: &str = "en";
const ALLOWED_LANGUAGES: &[&str] = &[DEFAULT_LANGUAGE, "ko", "de", "es", "fa", "pt", "ru", "tr"];

#[derive(Clone)]
pub struct I18n {
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let translations = Self::load_all_translations(dir)
            .unwrap_or_else(|e| panic!("translation load error: {e}"));

        I18n { translations }
    }

    fn load_all_translations<P: AsRef<Path>>(
        dir: P,
    ) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
        let mut all_translations = HashMap::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(lang) = path.file_stem().and_then(|s| s.to_str()) {
                    let translations = Self::load_and_flatten_translation(&path)?;
                    all_translations.insert(lang.to_string(), translations);
                }
            }
        }

        Ok(all_translations)
    }

    fn load_and_flatten_translation<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let json = serde_json::from_str(&content)?;
        let mut translation = HashMap::new();
        Self::flatten_json(&json, None, &mut translation);

        Ok(translation)
    }

    fn flatten_json(value: &Value, prefix: Option<String>, map: &mut HashMap<String, String>) {
        match value {
            Value::Object(obj) => {
                for (k, v) in obj {
                    let new_prefix = match &prefix {
                        Some(pref) => format!("{}.{}", pref, k),
                        None => k.to_string(),
                    };
                    Self::flatten_json(v, Some(new_prefix), map);
                }
            }
            Value::String(s) => {
                if let Some(key) = prefix {
                    map.insert(key, s.to_string());
                }
            }
            _ => {}
        }
    }

    pub fn translate_errors(
        &self,
        req: &HttpRequest,
        errors: &ValidationErrors,
    ) -> HashMap<String, Vec<String>> {
        let mut error_map = HashMap::new();

        for (field, field_errors) in errors.field_errors().iter() {
            let translated = field_errors
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| "parameter error".into())
                })
                .map(|msg| self.translate(req, &msg))
                .collect();

            error_map.insert(field.to_string(), translated);
        }

        error_map
    }

    pub fn translate(&self, req: &HttpRequest, msg: &str) -> String {
        let accept_language = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|h| h.to_str().ok());

        let preferred_language = Self::get_preferred_language(accept_language);

        self.translations
            .get(&preferred_language)
            .and_then(|map| map.get(msg).cloned())
            .unwrap_or_else(|| msg.to_string())
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
    use serde_json::json;

    use super::*;

    #[test]
    fn test_flatten_json() {
        let json_value = json!({
          "welcome": "Welcome",
          "required": "is required",
          "notFound": "has not been found",
          "duplicate": "is already in use",
          "typeMismatch": {
            "birthDate": "invalid date"
          }
        });

        let mut flat_map = HashMap::new();
        I18n::flatten_json(&json_value, None, &mut flat_map);

        assert_eq!(flat_map.get("required").unwrap(), "is required");
        assert_eq!(
            flat_map.get("typeMismatch.birthDate").unwrap(),
            "invalid date"
        );
    }

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
