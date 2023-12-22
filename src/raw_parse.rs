#[allow(unused_variables, dead_code)]
// use regex::{Regex};
pub struct RawParser {}

impl RawParser {
    // Return Vec<(start_ix, len)>
    #[allow(unused_variables, dead_code)]
    pub fn parse_lines(&self, log_text: &str) -> Vec<usize> {
        let mut list = Vec::new();
        // let foo = log_text.as_bytes();
        let mut prev_newline = true;
        let match_date = b"[dddd-dd-dd dd:dd:dd] ";
        // let mut match_iter = match_date.iter();
        // let mut starting_real_ix = 0;
        //

        let test_arr = log_text.as_bytes();
        for i in 0..(test_arr.len() - match_date.len()) {
            let c = test_arr[i];
            if prev_newline {
                let (got_match, offset) = self.match_date(&test_arr[i..]);
                if got_match {
                    list.push(i);
                }
            } else {
                prev_newline = c == b'\n' || c == b'\r' || i == 0;
            }
        }

        return list;        
    }

    pub fn match_date(&self, text: &[u8]) -> (bool, usize) {
        let mut ix = 0;
        let match_date = b"[dddd-dd-dd dd:dd:dd] ";
        let mut match_iter = match_date.iter();

        while let Some(mc) = match_iter.next() {
            let c = text[ix];
            ix += 1;
                // println!("In! '{c}'");
            let res = match mc {
                // 'n' => 
                b'd' => c >= 48 && c <= 57,
                 _ => *mc == c,
            };
                 

            if !res {
                return (false, ix);
            }
        }
    
        return (true, ix);
    }
}




#[cfg(test)]
mod tests {
    // use crate::{log_line };
    use super::RawParser;

    #[test]
    fn match_date_starty_of_line() {
        let short_log: &str = "[2023-02-14 13:43:49]  banan ding dong";

        let p = RawParser {};
        assert_eq!(
            p.match_date(&short_log.as_bytes()[0..]),
            (true, 22)
        );
    }

    #[test]
    fn match_date_middle_of_line_no_match() {
        let short_log: &str = "asbc[2023-02-14 13:43:49]  banan ding dong";

        let p = RawParser {};
        assert_eq!(
            p.match_date(short_log.as_bytes()),
            (false, 1)
        );
    }

    #[test]
    fn match_date_middle_of_line_match() {
        let short_log: &str = "
[2023-02-14 13:43:49]  banan ding dong";

        let p = RawParser {};
        assert_eq!(
            p.match_date(short_log.as_bytes()),
            (false, 1)
        );
    }



    #[test]
    fn exploration_line() {
        let short_log: &str = "[2023-02-14 13:43:49]  banan ding dong";

        let p = RawParser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![0]
        );
    }

 

    #[test]
    fn exploration() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 
[2023-02-14 13:43:50] local.INFO: Incoming webhook: 9 ";

        let p = RawParser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![0, 55, 110]
        );
    }

    #[test]
    fn log_with_two_lines() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
    Log line 2
Log line 3
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 ";

        let p = RawParser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![0, 81]
        );
    }
}

