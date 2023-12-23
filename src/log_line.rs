#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct LogLine {
    pub text: String,
    pub source: String,
    pub date: String,
    pub time: String,
}

impl LogLine {
    // pub fn new() -> Self {
    //     LogLine {
    //         title: "".to_owned(),
    //         source: "".to_owned(),
    //         date: "".to_owned(),
    //         time: "".to_owned(),
    //     }
    // }
    pub fn slug(&self) ->&str {
        &self.text[0..10]
    }
}

use self_cell::self_cell;

#[derive(Debug, Eq, PartialEq)]
pub struct LogLines<'a>(pub Vec<LogLine2<'a>>);

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

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct LogLine2<'a> {
    source: &'a str,
    log_level_len: usize,
    slug_len: usize,
}

impl<'a> LogLine2<'a> {

    pub fn parse(source: &'a str) -> Self {
        let ls = &source[22..].as_bytes();

        let mut lg_len: usize = 0;
        while let Some(c) = ls.get(lg_len) {
            if *c == b':' {
                break;
            }

            lg_len+=1;
        }

        LogLine2  {
            source,
            log_level_len: lg_len,
            slug_len: 10,
        }
    }

    pub fn text(&self) ->&str {
        let ix = 22+self.log_level_len + 2;
        &self.source[ix..]
    }

    pub fn slug(&self) ->&str {
        let ix = 22+self.log_level_len + 2;
        let end = usize::min(self.source.len(), ix+self.slug_len);
        &self.source[ix..end]
    }

    pub fn info(&self) ->&str {
        let ix = 22+self.log_level_len + 2;
        &self.source[0..ix]
    }

    pub fn date(&self) ->&str {
        &self.source[0..21]
    }

    pub fn log_level(&self) ->&str {
        &self.source[22..22+self.log_level_len]
    }
}
