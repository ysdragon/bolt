// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! HTML/XSS sanitization via ammonia

use ring_lang_rs::*;

/// bolt_sanitize_html(input) → string (strips dangerous tags, keeps safe ones)
ring_func!(bolt_sanitize_html, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let clean = ammonia::clean(input);
    ring_ret_string!(p, &clean);
});

/// bolt_sanitize_strict(input) → string (strips ALL HTML)
ring_func!(bolt_sanitize_strict, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let clean = ammonia::Builder::new()
        .tags(std::collections::HashSet::new())
        .clean(input)
        .to_string();
    ring_ret_string!(p, &clean);
});

/// bolt_escape_html(input) → string (escapes for HTML body context)
ring_func!(bolt_escape_html, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let escaped = html_escape::encode_text(input).to_string();
    ring_ret_string!(p, &escaped);
});

/// bolt_escape_attr(input) → string (safe for HTML attribute values, including unquoted)
ring_func!(bolt_escape_attr, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let escaped = html_escape::encode_unquoted_attribute(input).to_string();
    ring_ret_string!(p, &escaped);
});

/// bolt_escape_js(input) → string (safe for embedding in single-quoted JavaScript string literals)
ring_func!(bolt_escape_js, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let escaped = html_escape::encode_script_single_quoted_text(input).to_string();
    ring_ret_string!(p, &escaped);
});

/// bolt_escape_js_dq(input) → string (safe for embedding in double-quoted JavaScript string literals)
ring_func!(bolt_escape_js_dq, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let escaped = html_escape::encode_script_double_quoted_text(input).to_string();
    ring_ret_string!(p, &escaped);
});

/// bolt_escape_url(input) → string (URL-encode for safe embedding in URL query values)
ring_func!(bolt_escape_url, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);
    let input = ring_get_string!(p, 1);
    let encoded = urlencoding::encode(&input);
    ring_ret_string!(p, &encoded);
});

#[cfg(test)]
mod tests {
    /// True when `quote` never appears unescaped (always preceded by `\`).
    fn all_quotes_escaped(out: &str, quote: char) -> bool {
        out.char_indices()
            .filter(|(_, c)| *c == quote)
            .all(|(i, _)| i > 0 && out.as_bytes()[i - 1] == b'\\')
    }

    #[test]
    fn test_escape_js_single_quoted_neutralizes_quote() {
        let out = html_escape::encode_script_single_quoted_text("';alert(1);//").to_string();
        assert!(
            all_quotes_escaped(&out, '\''),
            "unescaped single quote in: {out}"
        );
        assert!(!out.contains("</script>"));
    }

    #[test]
    fn test_escape_js_dq_neutralizes_double_quote() {
        let out = html_escape::encode_script_double_quoted_text("\";alert(1);//").to_string();
        assert!(
            all_quotes_escaped(&out, '"'),
            "unescaped double quote in: {out}"
        );
        assert!(!out.contains("</script>"));
    }
}
