use gpui::{AppContext as _, Context, Entity, Window};
use gpui_component::input::InputState;

pub struct SearchState {
    pub query: String,
    pub case_sensitive: bool,
    /// Byte offsets of each match in the original content.
    pub match_offsets: Vec<usize>,
    pub current_index: usize,
    pub is_visible: bool,
    pub input_state: Entity<InputState>,
}

impl SearchState {
    pub fn new(window: &mut Window, cx: &mut Context<crate::app::AppView>) -> Self {
        let input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Rechercher..."));
        Self {
            query: String::new(),
            case_sensitive: false,
            match_offsets: Vec::new(),
            current_index: 0,
            is_visible: true,
            input_state,
        }
    }

    pub fn matches_count(&self) -> usize {
        self.match_offsets.len()
    }

    pub fn find_matches(&mut self, content: &str) {
        self.match_offsets.clear();

        if self.query.is_empty() {
            self.current_index = 0;
            return;
        }

        if self.case_sensitive {
            let mut start = 0;
            while let Some(pos) = content[start..].find(&self.query) {
                self.match_offsets.push(start + pos);
                start += pos + self.query.len();
            }
        } else {
            let lower_content = content.to_lowercase();
            let lower_query = self.query.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower_content[start..].find(&lower_query) {
                self.match_offsets.push(start + pos);
                start += pos + lower_query.len();
            }
        }

        if self.match_offsets.is_empty() {
            self.current_index = 0;
        } else if self.current_index >= self.match_offsets.len() {
            self.current_index = 0;
        }
    }

    pub fn next_match(&mut self) {
        if !self.match_offsets.is_empty() {
            self.current_index = (self.current_index + 1) % self.match_offsets.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.match_offsets.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.match_offsets.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }

    /// Returns a ratio (0.0 to 1.0) of where the current match is in the content.
    pub fn current_match_ratio(&self, content_len: usize) -> Option<f32> {
        if content_len == 0 || self.match_offsets.is_empty() {
            return None;
        }
        let offset = self.match_offsets[self.current_index];
        Some(offset as f32 / content_len as f32)
    }

    /// Produce a copy of `content` with search matches wrapped in backticks (inline code)
    /// so they render with a background highlight. The current match gets bold+code.
    /// Matches inside fenced code blocks or that contain backticks are skipped.
    pub fn highlighted_content(&self, content: &str) -> String {
        if self.query.is_empty() || self.match_offsets.is_empty() {
            return content.to_string();
        }

        let query_len = self.query.len();
        let fenced_regions = find_fenced_code_regions(content);

        let mut result = String::with_capacity(content.len() + self.match_offsets.len() * 4);
        let mut last_end = 0;

        for (i, &offset) in self.match_offsets.iter().enumerate() {
            let match_end = offset + query_len;
            let matched_text = &content[offset..match_end];

            // Skip matches inside fenced code blocks or containing backticks
            let in_fenced = fenced_regions
                .iter()
                .any(|&(start, end)| offset >= start && match_end <= end);
            if in_fenced || matched_text.contains('`') {
                continue;
            }

            result.push_str(&content[last_end..offset]);

            result.push('`');
            result.push_str(matched_text);
            result.push('`');

            last_end = match_end;
        }

        result.push_str(&content[last_end..]);
        result
    }
}

/// Find byte ranges of fenced code blocks (``` or ~~~) in the content.
fn find_fenced_code_regions(content: &str) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut fence_start = None;

    for (line_start, line) in line_byte_offsets(content) {
        let trimmed = line.trim_start();
        let is_fence = trimmed.starts_with("```") || trimmed.starts_with("~~~");
        if is_fence {
            if let Some(start) = fence_start {
                // Closing fence
                regions.push((start, line_start + line.len()));
                fence_start = None;
            } else {
                // Opening fence
                fence_start = Some(line_start);
            }
        }
    }

    // If a fence was opened but never closed, mark the rest as fenced
    if let Some(start) = fence_start {
        regions.push((start, content.len()));
    }

    regions
}

/// Iterate over lines with their byte offsets.
fn line_byte_offsets(content: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    content.split('\n').map(move |line| {
        let start = offset;
        offset += line.len() + 1; // +1 for the '\n'
        (start, line)
    })
}
