extern crate slack;
extern crate regex;

pub mod karma {
    use slack::RtmClient;
    use slack::Message;
    use slack::Message::Standard;
    use regex::Regex;

    pub fn handle_message(cli: &RtmClient, msg: Box<Message>) {
        let text = match *msg {
            Standard(std_msg) => {
                println!("Sender: {}", std_msg.user.unwrap());
                std_msg.text
            },
            _ => None,
        };

        match text {
            Some(text) => parse_recipients(&text[..]),
            None => println!("not standard message"),
        }
    }

    fn parse_recipients(text: &str) {
        let re = Regex::new(r"([[:alpha:]0-9]+)\+\+").unwrap();

        for capture in re.captures_iter(text) {
            println!("Captured: {} {}", &capture[0], &capture[1]);
        }
    }
}
