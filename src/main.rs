extern crate slack;
extern crate slack_api;
extern crate regex;
#[macro_use]
extern crate mysql;
#[macro_use]
extern crate lazy_static;

pub mod slackhandler;
pub mod data_layer;
pub mod schema;
pub mod karma;

use slack::RtmClient;
use slackhandler::SlackHandler;
use karma::KarmaManager;
use std::env;

fn main() {
    let api_key = match env::var("SLACK_BOT_TOKEN") {
        Ok(token) => token,
        Err(_)    => panic!("Failed to get SLACK_BOT_TOKEN from env"),
    };

    let mut handler = SlackHandler { karma_manager: KarmaManager::new() };
    println!("Running bot");
    RtmClient::login_and_run(&api_key, &mut handler)
        .expect("client failed to login and run");
}
