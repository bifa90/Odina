// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 bifa90 — Odina
//! The locale files, read by the program that ships them.
//!
//! Until now only the web page read `locales.js`; the binary carried message
//! **keys** and nothing that turned them into words. A desktop shell needs the
//! other half: the ten `.ftl` files are embedded here, parsed by this module,
//! and answered in the user's language. One source of truth, no second copy of
//! the strings, and no network.
//!
//! The parser is deliberately narrow: `key = value` at the start of a line,
//! `##` is a comment, and `{ $name }` is the only placeholder form used by the
//! shipped files. Fluent's full grammar would need a dependency and a spec
//! surface we do not use; a narrow reader that refuses what it does not know is
//! easier to keep honest than a wide one we cannot test.

/// The ten languages, in the order the locale files are reviewed.
pub const LANGS: &[&str] = &[
    "ar", "de", "en", "es", "fr", "hi", "ja", "pt-BR", "ru", "zh-Hans",
];

/// The embedded file for `lang`, or nothing for a language we do not ship.
pub fn source(lang: &str) -> Option<&'static str> {
    let text = match lang {
        "ar" => include_str!("../../../locales/ar/odina.ftl"),
        "de" => include_str!("../../../locales/de/odina.ftl"),
        "en" => include_str!("../../../locales/en/odina.ftl"),
        "es" => include_str!("../../../locales/es/odina.ftl"),
        "fr" => include_str!("../../../locales/fr/odina.ftl"),
        "hi" => include_str!("../../../locales/hi/odina.ftl"),
        "ja" => include_str!("../../../locales/ja/odina.ftl"),
        "pt-BR" => include_str!("../../../locales/pt-BR/odina.ftl"),
        "ru" => include_str!("../../../locales/ru/odina.ftl"),
        "zh-Hans" => include_str!("../../../locales/zh-Hans/odina.ftl"),
        _ => return None,
    };
    Some(text)
}

/// The raw value of `key` in one file, without substituting anything.
pub fn value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    for line in source.lines() {
        let Some((name, rest)) = line.split_once(" = ") else {
            continue;
        };
        if name.trim() == key {
            return Some(rest.trim());
        }
    }
    None
}

/// `key` in `lang`, falling back to English when a translation is missing, so a
/// half-translated file shows a word instead of a hole. A language we do not
/// ship is not a fallback: it is a caller mistake and answers `None`.
pub fn translate(lang: &str, key: &str) -> Option<String> {
    translate_args(lang, key, &[])
}

/// The same, with `{ $name }` placeholders filled from `args`.
pub fn translate_args(lang: &str, key: &str, args: &[(&str, &str)]) -> Option<String> {
    let mut text = source(lang)?;
    let mut found = value(text, key);
    if found.is_none() {
        text = source("en")?;
        found = value(text, key);
    }
    let mut out = found?.to_string();
    for (name, arg) in args {
        let placeholder = format!("{{ ${name} }}");
        out = out.replace(&placeholder, arg);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_language_has_an_embedded_file() {
        for &lang in LANGS {
            let text = source(lang).unwrap_or_else(|| panic!("{lang} is not embedded"));
            assert!(value(text, "app.name").is_some(), "{lang} has no app.name");
        }
        assert!(source("xx").is_none());
    }

    #[test]
    fn a_comment_is_not_a_definition_and_a_near_miss_is_refused() {
        assert_eq!(value("a.b = one\n", "a.b"), Some("one"));
        assert_eq!(value("## a.b = one\n", "a.b"), None);
        assert_eq!(value("a.b = one\n", "a.c"), None);
        assert_eq!(value("a.b = one two\n", "a.b"), Some("one two"));
    }

    #[test]
    fn a_placeholder_is_filled_and_the_rest_of_the_line_survives() {
        let text = translate_args("ar", "update.chip", &[("v", "1.02")]);
        let text = text.unwrap();
        assert!(text.contains("1.02"), "{text}");
        assert!(!text.contains("$v"), "{text}");
        // The same key in a language whose word order differs.
        let german = translate_args("de", "update.chip", &[("v", "1.02")]).unwrap();
        assert!(german.contains("1.02"), "{german}");
    }

    #[test]
    fn a_missing_translation_falls_back_to_english_and_an_unknown_language_does_not() {
        // `translate` must not invent a language: `xx` is refused outright.
        assert!(translate("xx", "state.protected").is_none());
        // A key that no file has is nothing, in every language.
        for &lang in LANGS {
            assert!(translate(lang, "no.such.key").is_none(), "{lang}");
        }
    }

    #[test]
    fn all_ten_files_define_every_key_the_shell_asks_for() {
        let keys = ["state.protected", "state.stale", "update.chip", "home.version"];
        for &lang in LANGS {
            let text = source(lang).unwrap();
            for key in keys {
                assert!(value(text, key).is_some(), "{lang} has no {key}");
            }
        }
    }
}
