// text.rs — text sanitisation helpers.

/// Strips Unity Rich Text markup tags from `s`, returning plain text.
///
/// Unity Rich Text uses HTML-like tags: `<b>`, `</b>`, `<color=#ff0000>`,
/// `</color>`, `<size=20>`, etc.  This function drops every `<…>` span
/// so only the visible characters remain.
///
/// Examples:
///   `<i><color=yellow>ILuv</color></i>` → `"ILuv"`
///   `Hello <b>world</b>`               → `"Hello world"`
///   `no tags here`                     → `"no tags here"`
pub fn strip_rich_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<'       => { in_tag = true; }
            '>' if in_tag => { in_tag = false; }
            _ if !in_tag  => { out.push(c); }
            _         => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_color_and_italic() {
        assert_eq!(
            strip_rich_text("<i><color=yellow>ILuv</color></i>"),
            "ILuv"
        );
    }

    #[test]
    fn strips_bold() {
        assert_eq!(strip_rich_text("Hello <b>world</b>"), "Hello world");
    }

    #[test]
    fn passthrough_plain() {
        assert_eq!(strip_rich_text("no tags here"), "no tags here");
    }

    #[test]
    fn empty_after_strip() {
        assert_eq!(strip_rich_text("<color=red></color>"), "");
    }
}
