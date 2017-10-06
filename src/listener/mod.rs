use regex::Captures;
use regex::Regex;
use slack::{User, RtmClient};

pub struct Message {
    pub user: User,
    pub text: String,
    pub is_addressed: bool,
    pub channel: String,
}

pub trait MessageListener {
    fn help(&self) -> String;
    fn handle(&self, message: &Message, cli: &RtmClient);
    fn re(&self) -> &Regex;

    fn can_handle(&self, msg: &Message) -> bool {
        self.re().is_match(&msg.text)
        && (!self.only_when_addressed() || msg.is_addressed)
    }

    fn get_captures<'a>(&self, msg: &'a Message) -> Option<Captures<'a>> {
        self.re().captures(&msg.text)
    }

    fn only_when_addressed(&self) -> bool {
        true
    }
}
