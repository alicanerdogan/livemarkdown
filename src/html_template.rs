const STYLES: &str = include_str!("../assets/index.css");
const SCRIPTS: &str = include_str!("../assets/index.js");

pub fn wrap_in_html_template(content: &str, title: Option<&str>) -> String {
    let title = title.unwrap_or("Markdown Document");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>{}</style>
</head>
<body>
<main>
{}
</main>
<script>
{}
</script>
</body>
</html>"#,
        title, STYLES, content, SCRIPTS
    )
}

pub fn get_styles() -> String {
    STYLES.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_with_default_title() {
        let content = "<h1>Test Content</h1>";
        let html = wrap_in_html_template(content, None);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Markdown Document</title>"));
        assert!(html.contains("<h1>Test Content</h1>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("</style>"));
    }

    #[test]
    fn test_wrap_with_custom_title() {
        let content = "<p>Some content</p>";
        let title = "My Custom Title";
        let html = wrap_in_html_template(content, Some(title));

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>My Custom Title</title>"));
        assert!(html.contains("<p>Some content</p>"));
    }

    #[test]
    fn test_html_structure() {
        let content = "<div>test</div>";
        let html = wrap_in_html_template(content, Some("Test"));

        // Check proper HTML5 structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains(r#"<html lang="en">"#));
        assert!(html.contains(r#"<meta charset="UTF-8">"#));
        assert!(html
            .contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#));
        assert!(html.contains("</html>"));
    }
}
