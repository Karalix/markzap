use std::fs;
use std::path::PathBuf;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::switch::Switch;
use gpui_component::text::TextView;
use gpui_component::{Icon, IconName};
use gpui_component::{h_flex, v_flex};

use crate::assets::AppIconName;
use crate::slidev;
use crate::state::AppMode;
use crate::views::presentation;

pub struct AppView {
    mode: AppMode,
    content: String,
    file_path: Option<PathBuf>,
    editor_state: Option<Entity<InputState>>,
    has_presentation: bool,
    _subscription: Option<Subscription>,
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
            _subscription: None,
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

        // Subscribe to text changes to keep self.content in sync and save to file
        let subscription = cx.subscribe(
            &state,
            |this: &mut Self, _entity, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    if let Some(ref state) = this.editor_state {
                        this.content = state.read(cx).value().to_string();
                        this.has_presentation = slidev::detect_presentation(&this.content);
                        this.save_to_file();
                        cx.notify();
                    }
                }
            },
        );

        self.editor_state = Some(state.clone());
        self._subscription = Some(subscription);
        state
    }

    /// Save current content to the source file, if one is associated.
    fn save_to_file(&self) {
        if let Some(ref path) = self.file_path {
            if let Err(e) = fs::write(path, &self.content) {
                eprintln!("Error saving file {:?}: {}", path, e);
            }
        }
    }

    /// Sync editor content into self.content when switching away from edit mode.
    fn sync_content_from_editor(&mut self, cx: &Context<Self>) {
        if let Some(ref state) = self.editor_state {
            self.content = state.read(cx).value().to_string();
            self.has_presentation = slidev::detect_presentation(&self.content);
        }
    }

    /// Replace the current document with the contents of the given file path.
    pub fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            format!(
                "# Error\n\nCould not read `{}`:\n\n```\n{}\n```",
                path.display(),
                e
            )
        });
        self.content = content;
        self.file_path = Some(path.clone());
        self.has_presentation = slidev::detect_presentation(&self.content);
        self.editor_state = None;
        self._subscription = None;
        self.mode = AppMode::Preview;
        if let Some(name) = path.file_name() {
            window.set_window_title(&format!("MarkZap \u{2014} {}", name.to_string_lossy()));
        }
        cx.notify();
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit = self.mode == AppMode::Edit;
        let has_presentation = self.has_presentation;

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
            div()
                .id("preview-scroll")
                .flex_1()
                .size_full()
                .p_4()
                .child(
                    TextView::markdown("md-preview", self.content.clone(), window, cx)
                        .scrollable(true)
                        .selectable(true),
                )
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
                    // Left spacer (for centering the switch)
                    .child(h_flex().w(px(150.)))
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
                                            this.mode = AppMode::Edit;
                                        } else {
                                            this.sync_content_from_editor(cx);
                                            this.mode = AppMode::Preview;
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
