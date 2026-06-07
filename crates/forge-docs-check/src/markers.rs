//! Shared `<!-- forge:* -->` marker-region helpers.
//!
//! Every generated-in-place block (API signatures, counts, example modules)
//! uses the same open/close HTML-comment markers. Centralizing extraction here
//! guarantees one critical property in one place: **line-ending normalization**.
//! On a Windows git checkout the docs may be CRLF while generated bodies are LF;
//! comparing raw bytes produced a phantom "stale" finding (a real bug caught only
//! by the Windows CI matrix). `find_region` normalizes CRLF→LF so every marker
//! consumer is line-ending-agnostic by construction.

/// Locate `open … close` in `page`. Returns `(open_start, close_end, body)`
/// where `body` is the text strictly between the markers, CRLF-normalized and
/// trimmed of the surrounding newlines. `None` if the open or close is absent.
pub fn find_region(page: &str, open: &str, close: &str) -> Option<(usize, usize, String)> {
    let open_start = page.find(open)?;
    let after_open = open_start + open.len();
    let close_rel = page[after_open..].find(close)?;
    let close_start = after_open + close_rel;
    let close_end = close_start + close.len();
    let body = page[after_open..close_start]
        .replace("\r\n", "\n")
        .trim_matches('\n')
        .to_string();
    Some((open_start, close_end, body))
}

/// Return `page` with the region's body replaced by `new_body`
/// (canonical `open\n{new_body}\n{close}` form), or `None` if there is no region.
pub fn replace_region(page: &str, open: &str, close: &str, new_body: &str) -> Option<String> {
    let (start, end, _) = find_region(page, open, close)?;
    let mut out = String::with_capacity(page.len());
    out.push_str(&page[..start]);
    out.push_str(open);
    out.push('\n');
    out.push_str(new_body);
    out.push('\n');
    out.push_str(close);
    out.push_str(&page[end..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: &str = "<!-- forge:x -->";
    const CLOSE: &str = "<!-- /forge:x -->";

    #[test]
    fn find_region_normalizes_crlf() {
        let page = format!("a\r\n{OPEN}\r\nBODY\r\nL2\r\n{CLOSE}\r\nb\r\n");
        let (_, _, body) = find_region(&page, OPEN, CLOSE).expect("region");
        assert_eq!(body, "BODY\nL2");
    }

    #[test]
    fn replace_region_rewrites_body_canonically() {
        let page = format!("pre\n{OPEN}\nold\n{CLOSE}\npost\n");
        let out = replace_region(&page, OPEN, CLOSE, "new").expect("region");
        assert_eq!(out, format!("pre\n{OPEN}\nnew\n{CLOSE}\npost\n"));
    }

    #[test]
    fn missing_region_returns_none() {
        assert!(find_region("no markers here", OPEN, CLOSE).is_none());
        assert!(replace_region("no markers", OPEN, CLOSE, "x").is_none());
    }
}
