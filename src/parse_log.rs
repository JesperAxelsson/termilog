use regex::Regex;

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

    pub fn parse_line_test(&self, log_text: &str) -> (String, String, String, String) {
        // let reg = Regex::new(r"^\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*)").expect("Failed parse regex");
        let reg = Regex::new(r"^(?s)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] ([\w\.]+):\s+(.*?)\n?\r?\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\]")
            .expect("Failed parse regex");

for caps  in reg.captures_iter(log_text){
println!("Log: {:?}", caps);
}

        let cap = reg.captures(log_text).expect("Failed to capture");
        return (
            cap[1].to_string(),
            cap[2].to_string(),
            cap[3].to_string(),
            cap[4].to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_log::Parser;

    

    #[test]
    fn exploration() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 ";

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
Log line 3".to_owned()
            )
        );
    }
}
