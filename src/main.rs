extern crate slack;
extern crate regex;

pub mod slackhandler;
pub mod karma;

use slack::RtmClient;
use slackhandler::SlackHandler;
use std::env;

fn main() {
    let api_key = match env::var("SLACK_BOT_TOKEN") {
        Ok(token) => token,
        Err(_)    => panic!("Failed to get SLACK_BOT_TOKEN from env"),
    };

    let mut handler = SlackHandler;
    RtmClient::login_and_run(&api_key, &mut handler)
        .expect("client failed to login and run");
}
