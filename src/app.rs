use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{self, *};
use gpui_component::ActiveTheme as _;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::switch::Switch;
use gpui_component::webview::WebView;
use gpui_component::{Icon, IconName, Sizable as _};
use gpui_component::{h_flex, v_flex};

use crate::assets::AppIconName;
use crate::render;
use crate::slidev;
use crate::state::AppMode;
use crate::views::presentation;

pub struct AppView {
    mode: AppMode,
    content: String,
    file_path: Option<PathBuf>,
    editor_state: Option<Entity<InputState>>,
    has_presentation: bool,
    dirty: bool,
    _subscription: Option<Subscription>,
    save_debounce: u64,
    preview_webview: Option<Entity<WebView>>,
}

impl AppView {
    pub fn new(content: String, file_path: Option<PathBuf>) -> Self {
        let has_presentation = slidev::detect_presentation(&content);
        Self {
            mode: AppMode::Preview,
            content,
            file_path,
            editor_state: None,
            has_presentation,
            dirty: false,
            _subscription: None,
            save_debounce: 0,
            preview_webview: None,
        }
    }

    /// Lazily create the editor state on first switch to Edit mode.
    fn ensure_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        if let Some(ref state) = self.editor_state {
            return state.clone();
        }

        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .default_value(&self.content)
        });

        // Subscribe to text changes to keep self.content in sync and debounce save to file
        let subscription = cx.subscribe(
            &state,
            |this: &mut Self, _entity, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    if let Some(ref state) = this.editor_state {
                        this.content = state.read(cx).value().to_string();
                        this.has_presentation = slidev::detect_presentation(&this.content);
                        cx.notify();

                        // Debounce save: increment generation counter and schedule a save
                        this.dirty = true;
                        this.save_debounce += 1;
                        let generation = this.save_debounce;
                        let entity = cx.entity().clone();
                        cx.spawn(async move |_, cx| {
                            cx.background_executor().timer(Duration::from_secs(2)).await;
                            cx.update(|cx| {
                                entity.update(cx, |this, cx| {
                                    if this.save_debounce == generation {
                                        this.save_to_file();
                                        this.dirty = false;
                                        cx.notify();
                                    }
                                });
                            })
                            .ok();
                        })
                        .detach();
                    }
                }
            },
        );

        self.editor_state = Some(state.clone());
        self._subscription = Some(subscription);
        state
    }

    /// Lazily create the preview WebView, loading the current content as HTML.
    fn ensure_preview_webview(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<WebView> {
        if let Some(ref wv) = self.preview_webview {
            return wv.clone();
        }

        let html = render::render_markdown_page(&self.content, cx.theme().is_dark());
        let wry_webview = wry::WebViewBuilder::new()
            .with_html(&html)
            .build_as_child(window)
            .expect("Failed to create preview WebView");

        let entity = cx.new(|cx| WebView::new(wry_webview, window, cx));
        self.preview_webview = Some(entity.clone());
        entity
    }

    /// Regenerate the HTML from the current content and reload it in the
    /// preview WebView (if it exists), then make it visible.
    fn refresh_preview(&self, cx: &mut Context<Self>) {
        if let Some(ref wv) = self.preview_webview {
            let html = render::render_markdown_page(&self.content, cx.theme().is_dark());
            wv.read(cx).load_html(&html).ok();
            wv.update(cx, |w, _| w.show());
        }
    }

    /// Save current content to the source file, if one is associated.
    pub fn save_to_file(&self) {
        if let Some(ref path) = self.file_path {
            if let Err(e) = fs::write(path, &self.content) {
                eprintln!("Error saving file {:?}: {}", path, e);
            }
        }
    }

    /// Sync editor content into self.content when switching away from edit mode.
    /// Also saves immediately (flush any pending debounced save).
    fn sync_content_from_editor(&mut self, cx: &Context<Self>) {
        if let Some(ref state) = self.editor_state {
            self.content = state.read(cx).value().to_string();
            self.has_presentation = slidev::detect_presentation(&self.content);
            self.save_to_file();
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit = self.mode == AppMode::Edit;
        let has_presentation = self.has_presentation;
        let dirty = self.dirty;

        // Build the content area depending on mode
        let content_area = if is_edit {
            let editor_state = self.ensure_editor(window, cx);
            div()
                .flex_1()
                .size_full()
                .child(
                    Input::new(&editor_state)
                        .h_full()
                        .w_full()
                        .font_family("Menlo")
                        .text_sm(),
                )
                .into_any_element()
        } else {
            let wv = self.ensure_preview_webview(window, cx);
            div()
                .flex_1()
                .size_full()
                .child(wv.clone())
                .into_any_element()
        };

        // Clone content for the presentation closure
        let content_for_presentation = self.content.clone();

        v_flex()
            .size_full()
            // Top bar
            .child(
                h_flex()
                    .w_full()
                    .h(px(48.))
                    .px_4()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    // Left: saving indicator (or spacer for centering the switch)
                    .child(h_flex().w(px(150.)).when(dirty, |this| {
                        this.child(
                            Icon::new(IconName::LoaderCircle)
                                .small()
                                .text_color(cx.theme().muted_foreground)
                                .with_animation(
                                    "saving-spinner",
                                    Animation::new(Duration::from_secs(1))
                                        .repeat()
                                        .with_easing(linear),
                                    |icon, delta| {
                                        icon.rotate(Radians(delta * std::f32::consts::TAU))
                                    },
                                ),
                        )
                    }))
                    // Center: mode switch
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(IconName::Eye)
                            .child(Switch::new("mode-switch").checked(is_edit).on_click({
                                let view = cx.entity().clone();
                                move |checked, _window, cx| {
                                    view.update(cx, |this, cx| {
                                        if *checked {
                                            // Hide the native WebView so it does
                                            // not overlay the editor.
                                            if let Some(ref wv) = this.preview_webview {
                                                wv.update(cx, |w, _| w.hide());
                                            }
                                            this.mode = AppMode::Edit;
                                        } else {
                                            this.sync_content_from_editor(cx);
                                            this.mode = AppMode::Preview;
                                            this.refresh_preview(cx);
                                        }
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(Icon::new(AppIconName::Pencil)),
                    )
                    // Right: presentation button (or spacer)
                    .child(
                        h_flex()
                            .w(px(150.))
                            .justify_end()
                            .when(has_presentation, |this| {
                                this.child(
                                    Button::new("presentation-btn")
                                        .icon(AppIconName::Presentation)
                                        .on_click({
                                            let content = content_for_presentation.clone();
                                            move |_ev, _window, cx| {
                                                let html =
                                                    slidev::generate_presentation_html(&content);
                                                presentation::open_presentation_window(html, cx);
                                            }
                                        }),
                                )
                            }),
                    ),
            )
            // Content area
            .child(content_area)
    }
}
