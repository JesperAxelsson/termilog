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
    pub source: &'a str,
    // pub time: &str,
}

impl<'a> LogLine2<'a> {
    // pub fn new() -> Self {
    //     LogLine {
    //         title: "".to_owned(),
    //         source: "".to_owned(),
    //         date: "".to_owned(),
    //         time: "".to_owned(),
    //     }
    // }
    pub fn slug(&self) ->&str {
        &self.source[22..]
    }

    pub fn date(&self) ->&str {
        &self.source[0..21]
    }

}
