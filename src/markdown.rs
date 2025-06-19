use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

pub fn render_to_html(markdown_content: &str) -> String {
    let mut options = ComrakOptions::default();

    // Enable source position tracking
    options.render.sourcepos = true;

    // Enable GitHub-flavored markdown extensions
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.header_ids = Some("".to_string());
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.front_matter_delimiter = Some("---".to_string());

    let plugins = ComrakPlugins::default();

    markdown_to_html_with_plugins(markdown_content, &options, &plugins)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_MARKDOWN: &str = include_str!("../examples/simple.md");
    const COMPLEX_MARKDOWN: &str = include_str!("../examples/complex.md");

    #[test]
    fn test_simple_markdown_rendering() {
        let html = render_to_html(SIMPLE_MARKDOWN);

        // Check that HTML is generated
        assert!(!html.is_empty());
        assert!(html.contains("<h1"));
        assert!(html.contains("data-sourcepos"));
    }

    #[test]
    fn test_complex_markdown_rendering() {
        let html = render_to_html(COMPLEX_MARKDOWN);

        // Check that HTML is generated with various elements
        assert!(!html.is_empty());
        assert!(html.contains("data-sourcepos"));

        // Should contain various markdown elements
        if COMPLEX_MARKDOWN.contains("# ") {
            assert!(html.contains("<h1"));
        }
        if COMPLEX_MARKDOWN.contains("- ") || COMPLEX_MARKDOWN.contains("* ") {
            assert!(html.contains("<ul") || html.contains("<li"));
        }
        if COMPLEX_MARKDOWN.contains("|") {
            assert!(html.contains("<table") || html.contains("<th") || html.contains("<td"));
        }
    }

    #[test]
    fn test_sourcepos_enabled() {
        let markdown = "# Test Header\n\nSome paragraph text.";
        let html = render_to_html(markdown);

        // Verify that sourcepos attributes are present
        assert!(html.contains("data-sourcepos"));
    }
}
