pub struct LogLine {
    pub title: String,
    pub source: String,
    pub date: String,
    pub time: String,
}

impl LogLine {
    pub fn new() -> Self {
        LogLine {
            title: "".to_owned(),
            source: "".to_owned(),
            date: "".to_owned(),
            time: "".to_owned(),
        }
    }
}
