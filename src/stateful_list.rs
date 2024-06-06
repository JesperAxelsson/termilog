use ratatui::widgets::ListState;
use std::{cmp, mem};

use crate::log_line::LogData;
use crate::log_line::LogLine;

pub struct StatefulList {
    pub state: ListState,
    pub items: LogData,
}

impl StatefulList {
    pub fn with_items(items: LogData) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn change_log_data(&mut self, log_data: LogData) {
        let data_len = log_data.len();
        self.items = log_data;
        if let Some(state) = self.state.selected() {
            *self.state.selected_mut() = Some(cmp::min(state, data_len));
        }
    }

    pub fn append_text(&mut self, content: &str) {
        let items = mem::replace(&mut self.items, LogData::empty());
        self.items = items.append_text(content);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn goto_start(&mut self) {
        if self.items.len() > 0 {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn goto_end(&mut self) {
        if self.items.len() > 0 {
            self.state.select(Some(self.items.len() - 1));
        } else {
            self.state.select(None);
        }
    }

    pub fn jump_relative(&mut self, jump: isize) {
        if self.items.len() > 0 {
            let curent_ix = self.state.offset() as isize;
            let ix = (curent_ix + jump).clamp(0, self.items.len() as isize - 1);
            self.state.select(Some(ix as usize));
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected_item(&mut self) -> Option<&LogLine> {
        let ix = self.state.selected();
        if let Some(ix) = ix {
            return Some(&self.items.log_lines()[ix]);
        }

        None
    }
}
