use gpui::*;
use gpui_component::Root;
use gpui_component::webview::WebView;

/// Opens a new window containing a WebView that renders the presentation HTML.
/// The window is sized to 2/3 of the primary display.
pub fn open_presentation_window(html: String, cx: &mut App) {
    let window_size = cx
        .primary_display()
        .map(|display| {
            let screen = display.bounds().size;
            size(screen.width * 2. / 3., screen.height * 2. / 3.)
        })
        .unwrap_or_else(|| size(px(1024.), px(768.)));

    if let Ok(window_handle) = cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("MarkZap Presentation".into()),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Fullscreen(Bounds::centered(
                None,
                window_size,
                cx,
            ))),
            focus: true,
            ..Default::default()
        },
        |window, cx| {
            let wry_webview = wry::WebViewBuilder::new()
                .with_html(&html)
                .build_as_child(window)
                .expect("Failed to create WebView");

            let webview_entity = cx.new(|cx| WebView::new(wry_webview, window, cx));

            cx.new(|cx| Root::new(webview_entity, window, cx))
        },
    ) {
        window_handle
            .update(cx, |_, window, _| {
                window.activate_window();
            })
            .ok();
    }
}
