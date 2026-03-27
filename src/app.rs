use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{self, *};
use gpui_component::ActiveTheme as _;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::switch::Switch;
use gpui_component::text::TextView;
use gpui_component::{Icon, IconName, Selectable as _, Sizable as _};
use gpui_component::{h_flex, v_flex};

use crate::ToggleSearch;
use crate::assets::AppIconName;
use crate::search::SearchState;
use crate::slidev;
use crate::state::AppMode;
use crate::views::presentation;

actions!(search, [NextMatch, PrevMatch, CloseSearch]);

pub struct AppView {
    mode: AppMode,
    content: String,
    file_path: Option<PathBuf>,
    editor_state: Option<Entity<InputState>>,
    has_presentation: bool,
    dirty: bool,
    _subscription: Option<Subscription>,
    _search_subscription: Option<Subscription>,
    save_debounce: u64,
    search: Option<SearchState>,
    preview_scroll: ScrollHandle,
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
            _search_subscription: None,
            save_debounce: 0,
            search: None,
            preview_scroll: ScrollHandle::new(),
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

    /// Lazily create the search state on first Cmd+F.
    fn ensure_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.search.is_some() {
            return;
        }

        let search = SearchState::new(window, cx);

        let subscription = cx.subscribe(
            &search.input_state,
            |this: &mut Self, _entity, event: &InputEvent, cx| {
                match event {
                    InputEvent::Change => {
                        if let Some(ref mut search) = this.search {
                            search.query = search.input_state.read(cx).value().to_string();
                            search.find_matches(&this.content);
                            search.current_index = 0;
                            cx.notify();
                        }
                    }
                    InputEvent::PressEnter { .. } => {
                        if let Some(ref mut search) = this.search {
                            search.next_match();
                            cx.notify();
                        }
                        this.scroll_to_current_match();
                    }
                    _ => {}
                }
            },
        );

        self.search = Some(search);
        self._search_subscription = Some(subscription);
    }

    /// Scroll the preview to the current search match using proportional estimation.
    fn scroll_to_current_match(&self) {
        let content_len = self.content.len();
        if let Some(ref search) = self.search {
            if let Some(ratio) = search.current_match_ratio(content_len) {
                let max = self.preview_scroll.max_offset();
                let y = -max.height * ratio;
                self.preview_scroll.set_offset(point(px(0.), y));
            }
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_edit = self.mode == AppMode::Edit;
        let has_presentation = self.has_presentation;
        let dirty = self.dirty;

        // Search bar visible?
        let search_visible = !is_edit
            && self
                .search
                .as_ref()
                .map_or(false, |s| s.is_visible);

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
            let preview_content = if search_visible {
                self.search.as_ref().unwrap().highlighted_content(&self.content)
            } else {
                self.content.clone()
            };

            div()
                .id("preview-scroll")
                .flex_1()
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&self.preview_scroll)
                .child(
                    div().p_4().child(
                        TextView::markdown("md-preview", preview_content, window, cx)
                            .scrollable(false)
                            .selectable(true),
                    ),
                )
                .into_any_element()
        };

        let search_bar = if search_visible {
            let search = self.search.as_ref().unwrap();
            let matches_count = search.matches_count();
            let current_index = search.current_index;
            let case_sensitive = search.case_sensitive;
            let indicator = if matches_count > 0 {
                format!("{}/{}", current_index + 1, matches_count)
            } else if search.query.is_empty() {
                String::new()
            } else {
                "0/0".to_string()
            };

            let view = cx.entity().clone();
            let view2 = cx.entity().clone();
            let view3 = cx.entity().clone();
            let view4 = cx.entity().clone();

            Some(
                h_flex()
                    .id("search-bar")
                    .key_context("SearchBar")
                    .w_full()
                    .h(px(40.))
                    .px_4()
                    .gap_2()
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .on_action(cx.listener(|this, _: &NextMatch, _window, cx| {
                        if let Some(ref mut search) = this.search {
                            search.next_match();
                            cx.notify();
                        }
                        this.scroll_to_current_match();
                    }))
                    .on_action(cx.listener(|this, _: &PrevMatch, _window, cx| {
                        if let Some(ref mut search) = this.search {
                            search.prev_match();
                            cx.notify();
                        }
                        this.scroll_to_current_match();
                    }))
                    .on_action(cx.listener(|this, _: &CloseSearch, _window, cx| {
                        if let Some(ref mut search) = this.search {
                            search.is_visible = false;
                            cx.notify();
                        }
                    }))
                    // Search icon
                    .child(
                        Icon::new(IconName::Search)
                            .small()
                            .text_color(cx.theme().muted_foreground),
                    )
                    // Search input
                    .child(
                        Input::new(&search.input_state)
                            .w(px(240.))
                            .small(),
                    )
                    // Match count indicator
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .min_w(px(40.))
                            .child(indicator),
                    )
                    // Prev match button
                    .child(
                        Button::new("search-prev")
                            .icon(IconName::ArrowUp)
                            .small()
                            .on_click(move |_ev, _window, cx| {
                                view.update(cx, |this, cx| {
                                    if let Some(ref mut search) = this.search {
                                        search.prev_match();
                                        cx.notify();
                                    }
                                    this.scroll_to_current_match();
                                });
                            }),
                    )
                    // Next match button
                    .child(
                        Button::new("search-next")
                            .icon(IconName::ArrowDown)
                            .small()
                            .on_click(move |_ev, _window, cx| {
                                view2.update(cx, |this, cx| {
                                    if let Some(ref mut search) = this.search {
                                        search.next_match();
                                        cx.notify();
                                    }
                                    this.scroll_to_current_match();
                                });
                            }),
                    )
                    // Case sensitivity toggle
                    .child(
                        Button::new("search-case")
                            .icon(IconName::CaseSensitive)
                            .small()
                            .selected(case_sensitive)
                            .on_click(move |_ev, _window, cx| {
                                view3.update(cx, |this, cx| {
                                    if let Some(ref mut search) = this.search {
                                        search.toggle_case();
                                        search.find_matches(&this.content);
                                        cx.notify();
                                    }
                                });
                            }),
                    )
                    // Close button
                    .child(
                        Button::new("search-close")
                            .icon(IconName::Close)
                            .small()
                            .on_click(move |_ev, _window, cx| {
                                view4.update(cx, |this, cx| {
                                    if let Some(ref mut search) = this.search {
                                        search.is_visible = false;
                                        cx.notify();
                                    }
                                });
                            }),
                    ),
            )
        } else {
            None
        };

        // Clone content for the presentation closure
        let content_for_presentation = self.content.clone();

        v_flex()
            .size_full()
            .on_action(cx.listener(|this, _: &ToggleSearch, window, cx| {
                if this.mode == AppMode::Preview {
                    this.ensure_search(window, cx);
                    if let Some(ref mut search) = this.search {
                        search.is_visible = !search.is_visible;
                        if search.is_visible {
                            search.input_state.update(cx, |state, cx| {
                                state.focus(window, cx);
                            });
                        }
                    }
                    cx.notify();
                }
            }))
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
                                            // Hide search when switching to Edit mode
                                            if let Some(ref mut search) = this.search {
                                                search.is_visible = false;
                                            }
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
            // Search bar (conditionally)
            .children(search_bar)
            // Content area
            .child(content_area)
    }
}
