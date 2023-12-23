use crate::log_line::{LogLine2, LogData, LogLines};

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
                let cc = c as char;
                // println!("Check new char: {cc}");

                let (got_match, offset) = self.match_date(&test_arr[i..]);
                if got_match {
                    // println!("Push index: {offset}");
                    list.push(i);
                }

                prev_newline = false;
            } else {
                let cc = c as char;
                prev_newline = c == b'\n' || c == b'\r' || i == 0;
                // println!("Check newline {cc} {prev_newline}");
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

    pub fn map_log(&self, log_text:  String, log_start:  Vec<usize>) -> LogData {

        let sl = [log_start.as_slice(), &[log_text.len()]].concat(); 

        let log_data = LogData::new(log_text, move |txt| {
            let mut log_lines = Vec::new();
            let mut pix = 0;
            let mut ix = 1;

            // for i in [1..log_text.len()] {
            while ix < sl.len()-1 {
                let start = sl[pix] as usize;
                let end = sl[ix];

                println!("Window: {start} to {end}");

                // let ll = LogLine2 {
                //     source: &log_text[0..2],
                // };

                // log_lines.push(ll);
                log_lines.push(&txt[start..end]);

                ix+=1;
                pix+=1;
            }
     
           LogLines( log_lines)

        });

        // println!("Window: {:?}", sl);
        //
        // let mut pix = 0;
        // let mut ix = 1;
        //
        // // for i in [1..log_text.len()] {
        // while ix < sl.len()-1 {
        //     let start = &sl[pix];
        //     let end = &sl[ix];
        //
        //     println!("Window: {start} to {end}");
        //
        //     let ll = LogLine2 {
        //         source: &log_text[0..2],
        //     };
        //
        //     log_lines.push(ll);
        //
        //     ix+=1;
        //     pix+=1;
        // }
        
        // for (start, end) in [log_start.as_slice(), &[log_text.len()]].concat().iter().windows(2) {
        //     println!("Window: {start} to {end}");
        // }

        // return log_lines;
        return log_data;
    }
}



#[cfg(test)]
mod tests {
    use crate::log_line::LogLine2;

    // use crate::{log_line };
    use super::RawParser;

    // #[test]
    // fn map_simple_string() {
    //     let short_log: &str = "[2023-02-14 13:43:49]  banan ding dong";
    //
    //     let p = RawParser {};
    //     let lines = p.parse_lines(&short_log);
    //     assert_eq!(
    //         p.map_log(short_log.to_string(), &lines),
    //         vec![LogLine2 {
    //             source: "[2023-02-14 13:43:49]  banan ding dong",
    //         }]
    //     );
    // }
    //

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

    #[test]
    fn log_with_two_lines_with_date() {
        let short_log: &str = "[2023-02-14 13:42:48] local.INFO: Incoming webhook: 7 
     [2023-02-14 13:43:49] Log line 2
Log line 3
[2023-02-14 13:43:49] local.INFO: Incoming webhook: 8 ";

        let p = RawParser {};
        assert_eq!(
            p.parse_lines(short_log),
            vec![0, 104]
        );
    }

}

