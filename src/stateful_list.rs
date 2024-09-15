use log::info;
use ratatui::widgets::ListState;
use std::{cmp, mem};

use crate::log_line::LogData;
use crate::log_line::LogLine;

pub struct StatefulList {
    /// Keeps track of UI list state
    pub state: ListState,

    /// Parsed log data
    items: LogData,

    /// Filter log data by these filters
    /// public because I'm lazy
    pub filter: Vec<String>,

    /// Filtered and ordered index of logs
    index_list: Vec<usize>,

    /// Skip the first <cutoff> logs
    cutoff: usize,
}

impl StatefulList {
    pub fn with_items(items: LogData) -> StatefulList {
        let index_list: Vec<usize> = (0..items.len()).collect();

        let mut lst = StatefulList {
            state: ListState::default(),
            index_list,
            items,
            filter: Vec::new(),
            cutoff: 0,
        };

        lst.update_ix_list();

        lst
    }

    /// Rerun filter and cutoff
    pub fn update_ix_list(&mut self) {
        self.index_list.clear();
        let ix_iter = self.items.log_lines().iter().enumerate().skip(self.cutoff);

        if self.filter.len() > 0 {
            for (ix, log) in ix_iter {
                if self.filter.iter().any(|pat| log.text().to_ascii_lowercase().contains(pat)) {
                    self.index_list.push(ix);
                }
            }
        } else {
            for (ix, _log) in ix_iter {
                self.index_list.push(ix);
            }
        }
    }

    /// Changes the current logdata, used when current log file is removed or
    /// cleared outside this program
    pub fn change_log_data(&mut self, log_data: LogData) {
        let data_len = log_data.len();
        self.items = log_data;
        if let Some(state) = self.state.selected() {
            *self.state.selected_mut() = Some(cmp::min(state, data_len));
        }
        self.update_ix_list();
    }

    /// Add new text to current log data
    pub fn append_text(&mut self, content: &str) {
        let items = mem::replace(&mut self.items, LogData::empty());
        self.items = items.append_text(content);
        // TODO: Consider filtering only new lines rather then all the lines
        self.update_ix_list();
    }

    pub fn get_cutoff(&mut self) -> usize {
        self.cutoff
    }

    pub fn set_cutoff(&mut self) {
        self.internal_set_cutoff(self.selected_index().unwrap_or_default());
    }

    pub fn reset_cutoff(&mut self) {
        self.internal_set_cutoff(0)
    }

    pub fn clear_all(&mut self) {
        self.unselect();
        self.internal_set_cutoff(self.items.len());
    }

    fn internal_set_cutoff(&mut self, cutoff: usize) {
        // TODO: When we change cutoff we want to keep current selected item if possible
        self.cutoff = cutoff;

        self.update_ix_list();
    }

    pub fn iter(&self) -> impl Iterator<Item = &LogLine<'_>> + '_ {
        self.index_list
            .iter()
            .map(|ix| self.items.borrow_dependent().0.get(*ix).unwrap())
    }

    pub fn next(&mut self) {
        if self.index_list.is_empty() {
            self.unselect();
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.index_list.len() - 1 {
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
        if self.index_list.is_empty() {
            self.unselect();
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.index_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn goto_start(&mut self) {
        if self.index_list.len() > 0 {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn goto_end(&mut self) {
        if self.index_list.len() > 0 {
            self.state.select(Some(self.index_list.len() - 1));
        } else {
            self.state.select(None);
        }
    }

    pub fn jump_relative(&mut self, jump: isize) {
        if self.index_list.len() > 0 {
            let curent_ix = self.state.offset() as isize;
            let ix = (curent_ix + jump).clamp(0, self.index_list.len() as isize - 1);
            self.state.select(Some(ix as usize));
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected_item(&mut self) -> Option<&LogLine> {
        let ix = self.selected_index();
        if let Some(ix) = ix {
            return Some(&self.items.log_lines()[ix]);
        }

        None
    }

    pub fn len(&self) -> usize {
        self.index_list.len()
    }

    pub fn inner_len(&self) -> usize {
        self.items.len()
    }

    fn selected_index(&self) -> Option<usize> {
        let ix = self.state.selected();
        if let Some(ix) = ix {
            return Some(self.index_list[ix]);
        }

        None
    }
}

// Filter stuff
// impl StatefulList {
//     pub fn add_filter(&mut self,
// }
