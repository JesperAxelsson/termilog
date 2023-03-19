use regex::{Captures, Regex};

use crate::log_line;

pub struct Parser {}

impl Parser {
    // pub fn parse_line(log_text: &str) -> log_line::LogLine {
    //     let reg = Regex::new("");

    //     log_line::LogLine {
    //         text: todo!(),
    //         source: todo!(),
    //         date: todo!(),
    //         time: todo!(),
    //     };
    // }

    // pub fn parse_line_test(&self, log_text: &str) -> (String, String, String, String) {
    //     // let reg = Regex::new(r"^\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*)").expect("Failed parse regex");
    //     let reg = Regex::new(r"^(?s)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*?)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\]")
    //         .expect("Failed parse regex");

    //     for caps in reg.captures_iter(log_text) {
    //         println!("Log: {:?}", caps);
    //     }

    //     let cap = reg.captures(log_text).expect("Failed to capture");
    //     return (
    //         cap[1].to_string(),
    //         cap[2].to_string(),
    //         cap[3].to_string(),
    //         cap[4].to_string(),
    //     );
    // }

    pub fn parse_line_test(&self, log_text: &str) -> (String, String, String, String) {
        let cap = self
            .parse_line_caps(log_text)
            .expect("Failed to capture regex");

        // for line in cap.iter() {
        //     println!("Line: {:?}", line.unwrap());

        //     for cap in line.unwrap(). {
        //         println!("cap: {:?}", cap );
        //     }
        // }

        return (
            cap[1].to_string(),
            cap[2].to_string(),
            cap[3].to_string(),
            cap[4].to_string(),
        );
    }

    // pub fn parse_line_caps<'t>(&self, log_text: &'t str) -> Option<(Captures<'t>, CaptureLocations)> {
    pub fn parse_line_caps<'t>(&self, log_text: &'t str) -> Option<Captures<'t>> {
        // let reg = Regex::new(r"^\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*)").expect("Failed parse regex");
        let reg = Regex::new(r"^(?s)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*?)(?:\n?\r?\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\]|$|\z)")
        // let reg = Regex::new(r"(^(?s)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*?))+(?:\n?\r?\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\]|$|\z)")
            .expect("Failed parse regex");

        let cap = reg.captures(log_text);
        // let loca = reg.capture_locations(log_text);
        return cap;
    }

    pub fn parse_lines(&self, log_text: &str) -> Vec<log_line::LogLine> {
        let mut result = Vec::new();
        let mut offset = 0;

        println!("Logtext: '{}'", log_text.len());

        while let Some(cap) = self.parse_line_caps(&log_text[offset..]) {
            // This is sooo fragile, but works!
            let mat = cap.get(4);
            offset += mat.unwrap().end();
            println!("Offset: {} log end '{}'", offset, &log_text[offset..]);
            result.push(log_line::LogLine {
                text: cap[4].to_owned(),
                source: cap[3].to_owned(),
                date: cap[1].to_owned(),
                time: cap[2].to_owned(),
            })
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::{log_line, parse_log::Parser};

    #[test]
    fn exploration() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 
[2023-02-14 13:43:50] local.INFO: Incoming webhook: 9 ";

        let p = Parser {};
        assert_eq!(
            p.parse_line_test(short_log),
            (
                "2023-02-14".to_owned(),
                "13:42:48".to_owned(),
                "local.INFO".to_owned(),
                "Incoming webhook: 7 ".to_owned()
            )
        );
    }

    #[test]
    fn log_with_two_lines() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
    Log line 2
Log line 3
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 ";

        let p = Parser {};
        assert_eq!(
            p.parse_line_test(short_log),
            (
                "2023-02-14".to_owned(),
                "13:42:48".to_owned(),
                "local.INFO".to_owned(),
                "Incoming webhook: 7 
    Log line 2
Log line 3"
                    .to_owned()
            )
        );
    }

    #[test]
    fn log_parse_many_lines() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
    Log line 2
Log line 3
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 9 ";

        let p = Parser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![
                log_line::LogLine {
                    date: "2023-02-14".to_owned(),
                    time: "13:42:48".to_owned(),
                    source: "local.INFO".to_owned(),
                    text: "Incoming webhook: 7 
    Log line 2
Log line 3"
                        .to_owned()
                },
                log_line::LogLine {
                    date: "2023-02-14".to_owned(),
                    time: "13:43:49".to_owned(),
                    source: "local.INFO".to_owned(),
                    text: "Incoming webhook: 8 ".to_owned()
                },
                log_line::LogLine {
                    date: "2023-02-14".to_owned(),
                    time: "13:43:49".to_owned(),
                    source: "local.INFO".to_owned(),
                    text: "Incoming webhook: 9 ".to_owned()
                }
            ]
        );
    }

    #[test]
    fn log_parse_one_line() {
        let short_log: &str = "[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8";

        let p = Parser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![log_line::LogLine {
                date: "2023-02-14".to_owned(),
                time: "13:43:49".to_owned(),
                source: "local.INFO".to_owned(),
                text: "Incoming webhook: 8".to_owned()
            }]
        );
    }
}
