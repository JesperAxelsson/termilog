use self_cell::self_cell;

use crate::raw_parse;

#[derive(Debug, Eq, PartialEq)]
pub struct LogLines<'a>(pub Vec<LogLine<'a>>);

// #[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
self_cell!(
    pub struct LogData {
        owner: String,
        // pub time: &str,

        #[covariant]
        dependent: LogLines,
    }

    impl {Debug, Eq, PartialEq}
);

impl LogData {
    pub fn empty() -> Self {
        LogData::new(String::new(), |_| { LogLines(Vec::new()) }) 
    }

     pub fn from_content(new_text: String) -> Self {
        let parser = raw_parse::RawParser {};
        let log_lines = parser.parse_lines(&new_text);

        let ll = parser.map_log(new_text, log_lines.clone());
        ll
    }

   pub fn append_text(self, new_text: &str) -> Self {
        let mut owner = self.into_owner();
        owner.push_str(new_text);

        let parser = raw_parse::RawParser {};
        let log_lines = parser.parse_lines(&owner);

        let ll = parser.map_log(owner, log_lines.clone());
        ll
    }

    pub fn len(&self) -> usize {
        self.log_lines().len()
    }

    pub fn log_lines(&self) -> &Vec<LogLine> {
        &self.borrow_dependent().0
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct LogLine<'a> {
    source: &'a str,
    log_level_len: usize,
}

impl<'a> LogLine<'a> {

    pub fn parse(source: &'a str) -> Self {
        let ls = &source[22..].as_bytes();

        let mut lg_len: usize = 0;
        while let Some(c) = ls.get(lg_len) {
            if *c == b':' {
                break;
            }

            lg_len+=1;
        }

        LogLine {
            source,
            log_level_len: lg_len,
        }
    }

    pub fn text(&self) ->&str {
        let ix = 22+self.log_level_len + 2;
        &self.source[ix..]
    }

    pub fn slug(&self, slug_len: usize) ->&str {
        let ix = 22+self.log_level_len + 2;
        let end = usize::min(self.source.len(), ix+slug_len);
        &self.source[ix..end]
    }

    pub fn info(&self) ->&str {
        let ix = 22+self.log_level_len + 2;
        &self.source[0..ix]
    }

    #[allow(dead_code)]
    pub fn date(&self) ->&str {
        &self.source[0..21]
    }

    #[allow(dead_code)]
    pub fn log_level(&self) ->&str {
        &self.source[22..22+self.log_level_len]
    }
}
