/// Detects whether markdown content looks like a slide presentation.
///
/// Heuristics:
/// 1. Starts with YAML frontmatter containing Slidev-related keys (theme, layout, class, slidev)
/// 2. Contains 3 or more `---` horizontal rule separators on their own line
pub fn detect_presentation(content: &str) -> bool {
    // Check for Slidev-style frontmatter
    if content.starts_with("---") {
        let rest = &content[3..];
        if let Some(end) = rest.find("---") {
            let frontmatter = rest[..end].to_lowercase();
            if frontmatter.contains("theme:")
                || frontmatter.contains("class:")
                || frontmatter.contains("layout:")
                || frontmatter.contains("slidev")
            {
                return true;
            }
        }
    }

    // Count --- separators on their own line
    let separator_count = content.lines().filter(|line| line.trim() == "---").count();
    separator_count >= 3
}

/// Generates a self-contained HTML page that renders the markdown as a slide
/// presentation using Reveal.js loaded from CDN.
pub fn generate_presentation_html(markdown_content: &str) -> String {
    // Escape markdown for safe embedding in a JS template literal
    let escaped = markdown_content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
        .replace("</script>", "<\\/script>");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>MarkZap Presentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@5/dist/reveal.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@5/dist/theme/white.css">
    <style>
        body {{ margin: 0; padding: 0; overflow: hidden; }}
        .reveal {{ height: 100vh; }}
    </style>
</head>
<body>
    <div class="reveal">
        <div class="slides" id="slides"></div>
    </div>
    <script src="https://cdn.jsdelivr.net/npm/reveal.js@5/dist/reveal.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/reveal.js@5/plugin/markdown/markdown.js"></script>
    <script>
        const markdown = `{escaped}`;
        const parts = markdown.split(/\n---\n/);
        const container = document.getElementById('slides');
        for (const part of parts) {{
            const trimmed = part.trim();
            if (!trimmed) continue;
            const section = document.createElement('section');
            section.setAttribute('data-markdown', '');
            const textarea = document.createElement('textarea');
            textarea.setAttribute('data-template', '');
            textarea.textContent = trimmed;
            section.appendChild(textarea);
            container.appendChild(section);
        }}
        Reveal.initialize({{
            plugins: [RevealMarkdown],
            hash: true,
            controls: true,
            progress: true,
        }});
    </script>
</body>
</html>"#
    )
}
