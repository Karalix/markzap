use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::*;
use gpui_component::Root;

mod app;
mod assets;
mod slidev;
mod state;
mod views;

actions!(markzap, [OpenFile, Quit]);

/// Convert a file:// URL string to a PathBuf.
fn url_to_path(url: &str) -> Option<PathBuf> {
    let path_str = url.strip_prefix("file://")?;
    let decoded = percent_decode(path_str);
    Some(PathBuf::from(decoded))
}

/// Simple percent-decoding for file paths (e.g. %20 -> space).
fn percent_decode(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).into_owned()
}

fn load_file(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file {:?}: {}", path, e);
        format!(
            "# Error\n\nCould not read `{}`:\n\n```\n{}\n```",
            path.display(),
            e
        )
    })
}

fn open_window(
    content: String,
    path: Option<PathBuf>,
    cx: &mut App,
) -> Option<(AnyWindowHandle, Entity<app::AppView>)> {
    let title = match path.as_ref().and_then(|p| p.file_name()) {
        Some(name) => format!("MarkZap — {}", name.to_string_lossy()),
        None => "MarkZap".to_string(),
    };

    let app_view: Rc<RefCell<Option<Entity<app::AppView>>>> = Rc::new(RefCell::new(None));
    let app_view_capture = app_view.clone();

    let window_handle = cx
        .open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    ..Default::default()
                }),
                is_resizable: true,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(
                        px(720.),
                        cx.primary_display()
                            .map(|display| display.bounds().size.height)
                            .unwrap_or_else(|| px(1080.)),
                    ),
                    cx,
                ))),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|_| app::AppView::new(content, path));
                *app_view_capture.borrow_mut() = Some(view.clone());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .ok()?;

    let view = app_view.borrow_mut().take()?;

    // Quit the app only when the last window is closed, and flush pending saves
    let view_for_close = view.clone();
    window_handle
        .update(cx, |_, window, cx| {
            window.on_window_should_close(cx, move |_, cx| {
                view_for_close.update(cx, |this, _cx| {
                    this.save_to_file();
                });
                // Count remaining windows (including this one, which hasn't closed yet)
                if cx.windows().len() <= 1 {
                    cx.quit();
                }
                true
            });
        })
        .ok();
    Some((window_handle.into(), view))
}

fn main() {
    // Parse CLI: first argument is the .md file path
    let args: Vec<String> = env::args().collect();
    let cli_file_path = args.get(1).map(PathBuf::from);

    // Shared buffer for file URLs received via macOS open events (double-click on .md).
    // macOS sends application:openURLs: which may arrive before applicationDidFinishLaunching,
    // so we buffer them here and drain in the run() closure.
    let pending_urls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let pending_for_callback = pending_urls.clone();

    let app = Application::new().with_assets(assets::Assets);

    // Register the open-urls handler BEFORE run().
    // This captures file:// URLs from macOS document open events.
    // URLs are buffered and picked up either at launch or by a polling timer.
    app.on_open_urls(move |urls: Vec<String>| {
        pending_for_callback.borrow_mut().extend(urls);
    });

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);

        cx.set_menus(vec![Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("Open\u{2026}", OpenFile),
                MenuItem::separator(),
                MenuItem::action("Quit MarkZap", Quit),
            ],
        }]);

        cx.bind_keys([
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);

        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        // Check for URLs received via macOS open events (e.g. double-click on .md file)
        let open_event_urls: Vec<String> = pending_urls.borrow_mut().drain(..).collect();

        let result = if let Some(path) = open_event_urls.first().and_then(|u| url_to_path(u)) {
            // Opened via macOS file association
            let content = load_file(&path);
            open_window(content, Some(path), cx)
        } else if let Some(ref path) = cli_file_path {
            // Opened via CLI argument
            let content = load_file(path);
            open_window(content, cli_file_path.clone(), cx)
        } else {
            // No file specified — show welcome screen
            let content = String::from(
                "# Welcome to MarkZap\n\n\
                 Open a `.md` file by passing it as a command-line argument:\n\n\
                 ```\n\
                 markzap path/to/file.md\n\
                 ```\n\n\
                 Or double-click a `.md` file to open it with MarkZap.",
            );
            open_window(content, None, cx)
        };

        // Register the OpenFile action (Cmd-O) — opens file in a new window
        cx.on_action(move |_: &OpenFile, cx| {
            let receiver = cx.prompt_for_paths(PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: None,
            });
            cx.spawn(async move |cx| {
                if let Ok(Ok(Some(paths))) = receiver.await {
                    if let Some(path) = paths.into_iter().next() {
                        cx.update(|cx| {
                            let content = load_file(&path);
                            open_window(content, Some(path), cx);
                        })
                        .ok();
                    }
                }
            })
            .detach();
        });

        // Poll for new open-url events that arrive after launch.
        // macOS on_open_urls doesn't give us access to cx, so we poll the buffer.
        let pending_urls_poll = pending_urls.clone();
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;
                let urls: Vec<String> = pending_urls_poll.borrow_mut().drain(..).collect();
                if !urls.is_empty() {
                    for url in urls {
                        if let Some(path) = url_to_path(&url) {
                            let content = load_file(&path);
                            cx.update(|cx| {
                                open_window(content, Some(path), cx);
                            })
                            .ok();
                        }
                    }
                }
            }
        })
        .detach();

        // Ignore the result — we no longer tie actions to a single window
        let _ = result;
    });
}
