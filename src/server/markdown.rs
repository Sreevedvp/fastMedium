//! Only parser-generated markup reaches the reader. Raw author HTML is escaped.
use pulldown_cmark::{html, Event, Options, Parser, Tag};

fn safe_url(url: &str, image: bool) -> bool {
    let value = url.trim();
    if value.chars().any(|c| c.is_control()) {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("https://")
        || lower.starts_with("http://")
        || (!image && lower.starts_with("mailto:"))
    {
        return true;
    }
    // Relative links cannot introduce a scheme, including encoded/obfuscated ones.
    !value
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .contains(':')
        && !value.starts_with('\\')
}
pub fn render(content: &str) -> String {
    let events = Parser::new_ext(
        content,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
    )
    .map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: if safe_url(&dest_url, false) {
                dest_url
            } else {
                "".into()
            },
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: if safe_url(&dest_url, true) {
                dest_url
            } else {
                "".into()
            },
            title,
            id,
        }),
        event => event,
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn author_html_is_text_and_markdown_survives() {
        let output = render(
            "# Heading\n\n**bold** <img src=x onerror=alert(1)>\n\n<script>alert(1)</script>",
        );
        assert!(output.contains("<h1>Heading</h1>") && output.contains("<strong>bold</strong>"));
        assert!(!output.contains("<script") && !output.contains("<img src=x"));
    }
    #[test]
    fn dangerous_destinations_are_removed() {
        for input in [
            "[click](javascript:alert%281%29)",
            "[click](JaVaScRiPt:alert%281%29)",
            "[click](javascript&#58;alert%281%29)",
            "![image](data:text/html,payload)",
            "[x](vbscript:msgbox)",
        ] {
            let output = render(input);
            assert!(output.contains("=\"\""), "{output}");
        }
        assert!(render("[safe](https://example.org) ![pic](/image.png)")
            .contains("href=\"https://example.org\""));
        assert!(!safe_url("java\nscript:alert(1)", false));
    }
}
