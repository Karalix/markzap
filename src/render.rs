use comrak::{Options, markdown_to_html};

/// Parses Markdown to HTML (via comrak) and wraps it in a self-contained,
/// themed HTML document suitable for display in a WebView.
///
/// The page includes an embedded, offline find bar bound to Cmd/Ctrl+F that
/// uses `window.find` (Enter = next, Shift+Enter = previous, Esc = close).
pub fn render_markdown_page(markdown: &str, dark: bool) -> String {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.superscript = true;
    // Allow raw HTML embedded in the Markdown (this is a viewer).
    options.render.unsafe_ = true;

    let body = markdown_to_html(markdown, &options);

    // Theme palette
    let (bg, fg, muted, border, code_bg, code_fg, quote_border, link) = if dark {
        (
            "#1e1e1e", "#e6e6e6", "#9a9a9a", "#3a3a3a", "#2a2a2a", "#e6e6e6", "#4a4a4a", "#5aa9e6",
        )
    } else {
        (
            "#ffffff", "#1f2328", "#656d76", "#d0d7de", "#f4f4f4", "#1f2328", "#d0d7de", "#0969da",
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>MarkZap</title>
<style>
  :root {{ color-scheme: {scheme}; }}
  html, body {{ margin: 0; padding: 0; }}
  body {{
    background: {bg};
    color: {fg};
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
  }}
  .markzap-content {{
    max-width: 860px;
    margin: 0 auto;
    padding: 32px 40px 64px;
    word-wrap: break-word;
  }}
  .markzap-content h1, .markzap-content h2 {{
    border-bottom: 1px solid {border};
    padding-bottom: .3em;
  }}
  .markzap-content h1, .markzap-content h2, .markzap-content h3,
  .markzap-content h4, .markzap-content h5, .markzap-content h6 {{
    margin-top: 1.6em;
    margin-bottom: .6em;
    line-height: 1.25;
    font-weight: 600;
  }}
  .markzap-content h1 {{ font-size: 2em; }}
  .markzap-content h2 {{ font-size: 1.5em; }}
  .markzap-content a {{ color: {link}; text-decoration: none; }}
  .markzap-content a:hover {{ text-decoration: underline; }}
  .markzap-content p, .markzap-content ul, .markzap-content ol {{ margin: 0 0 1em; }}
  .markzap-content li {{ margin: .25em 0; }}
  .markzap-content img {{ max-width: 100%; height: auto; }}
  .markzap-content code {{
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: .88em;
    background: {code_bg};
    color: {code_fg};
    padding: .2em .4em;
    border-radius: 4px;
  }}
  .markzap-content pre {{
    background: {code_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 14px 16px;
    overflow: auto;
  }}
  .markzap-content pre code {{
    background: transparent;
    padding: 0;
    font-size: .85em;
  }}
  .markzap-content blockquote {{
    margin: 0 0 1em;
    padding: 0 1em;
    color: {muted};
    border-left: .25em solid {quote_border};
  }}
  .markzap-content table {{
    border-collapse: collapse;
    margin: 0 0 1em;
    display: block;
    overflow: auto;
  }}
  .markzap-content th, .markzap-content td {{
    border: 1px solid {border};
    padding: 6px 13px;
  }}
  .markzap-content th {{ background: {code_bg}; font-weight: 600; }}
  .markzap-content hr {{
    border: 0;
    border-top: 1px solid {border};
    margin: 2em 0;
  }}
  .markzap-content ul.contains-task-list {{ list-style: none; padding-left: 1em; }}
  #markzap-find {{
    position: fixed;
    top: 12px;
    right: 16px;
    display: none;
    align-items: center;
    gap: 6px;
    background: {bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 6px 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,.25);
    z-index: 9999;
  }}
  #markzap-find input {{
    background: {code_bg};
    color: {fg};
    border: 1px solid {border};
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 13px;
    outline: none;
    width: 200px;
  }}
  #markzap-find button {{
    background: transparent;
    color: {muted};
    border: none;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
  }}
  #markzap-find button:hover {{ color: {fg}; }}
</style>
</head>
<body>
<div id="markzap-find">
  <input type="text" id="markzap-find-input" placeholder="Rechercher…" autocomplete="off">
  <button id="markzap-find-prev" title="Précédent">↑</button>
  <button id="markzap-find-next" title="Suivant">↓</button>
  <button id="markzap-find-close" title="Fermer">✕</button>
</div>
<div class="markzap-content">
{body}
</div>
<script>
(function() {{
  var bar = document.getElementById('markzap-find');
  var input = document.getElementById('markzap-find-input');
  function openBar() {{
    bar.style.display = 'flex';
    input.focus();
    input.select();
  }}
  function closeBar() {{
    bar.style.display = 'none';
    window.getSelection().removeAllRanges();
  }}
  function find(backwards) {{
    var q = input.value;
    if (!q) return;
    window.find(q, false, backwards, true, false, false, false);
  }}
  document.addEventListener('keydown', function(e) {{
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {{
      e.preventDefault();
      openBar();
    }} else if (e.key === 'Escape') {{
      closeBar();
    }}
  }});
  input.addEventListener('keydown', function(e) {{
    if (e.key === 'Enter') {{
      e.preventDefault();
      find(e.shiftKey);
    }} else if (e.key === 'Escape') {{
      e.preventDefault();
      closeBar();
    }}
  }});
  document.getElementById('markzap-find-next').addEventListener('click', function() {{ find(false); }});
  document.getElementById('markzap-find-prev').addEventListener('click', function() {{ find(true); }});
  document.getElementById('markzap-find-close').addEventListener('click', closeBar);
}})();
</script>
</body>
</html>"#,
        scheme = if dark { "dark" } else { "light" },
        bg = bg,
        fg = fg,
        muted = muted,
        border = border,
        code_bg = code_bg,
        code_fg = code_fg,
        quote_border = quote_border,
        link = link,
        body = body,
    )
}
