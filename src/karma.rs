extern crate mysql;

use slack::{Message, Event, RtmClient, User};
use slack::Message::Standard;
use slack_api::MessageStandard;
use regex::Regex;
use data_layer::DbManager;
use schema::{KarmaRecord, UserRecord};
use std::collections::HashMap;

pub struct KarmaManager {
    db_manager: DbManager,
}

impl KarmaManager {
    pub fn new() -> KarmaManager {
        println!("Initializing new `KarmaManager`...");

        KarmaManager { db_manager: DbManager::new() }
    }

    pub fn handle_message(&self, cli: &RtmClient, msg: Box<Message>) {
        println!("MESSAGE: {:?}", msg);
        match *msg {
            Standard(std_msg) => self.handle_std_msg(cli, &std_msg),
            _ => return,
        }
    }

    pub fn handle_std_msg(&self, cli: &RtmClient, msg: &MessageStandard) {
        let text = &msg.text;

        let recipients = match *text {
            Some(ref text) => Some(self.parse_recipients(&text[..])),
            None => None,
        };

        // TODO need to convert records to KarmaRecords and then write them
        // to the database. I might not even have to fetch the database users
        // since we're dealing with just IDs now, but I should make sure that
        // the database user does exist. Slack escapes the '<' and '>'
        // characters so I don't need to worry about someone trying to spoof
        // a user ID and inject SQL or break my regex.
        match recipients {
            Some(ref recipients) if !recipients.is_empty() => {
                let db_users = self.db_manager.fetch_db_user_list();
                let mut records: Vec<&UserRecord> = Vec::new();

                records = db_users.iter().filter(|db_user| {
                    let mut recipient = false;

                    recipients.iter().for_each(|ref tup| {
                        let slack_id = db_user.slack_id
                            .clone().unwrap_or("".to_string());

                        if slack_id == tup.0 {
                            recipient = true;
                        }
                    });

                    recipient
                }).collect();

                println!("RECIPIENT: {:?}", recipients);
                println!("RECORDS: {:?}", records);
            },
            Some(_) => (),
            None => (),
        }
    }

    // TODO implement when API fires reaction events
    pub fn handle_reaction(&self, cli: &RtmClient, event: &Event) {
        // Not implemented
        match event {
            &Event::ReactionAdded { .. } => (),
            _ => return,
        }
    }

    // Parses each increment/decrement applied to a user and returns a
    // (String, i32) tuple representing (user, points).
    fn parse_recipients(&self, text: &str) -> Vec<(String, i32)> {
        let mut recipient_map = HashMap::new();

        for capture in INC_RE.captures_iter(text) {
            *recipient_map.entry(String::from(&capture[1])).or_insert(0) += 1;
        }

        for capture in DEC_RE.captures_iter(text) {
            *recipient_map.entry(String::from(&capture[1])).or_insert(0) += -1;
        }

        recipient_map.iter()
            .map(|(name, points)| (name.clone(), points.clone())).collect()
    }

    pub fn update_user(&self, cli: &RtmClient, user: &User) {
        let user_vec = vec![
            UserRecord {
                id: None,
                slack_id: user.id.clone(),
                deleted: user.deleted.unwrap_or_default(),
            },
        ];

        self.db_manager.update_users(&user_vec);
    }

    pub fn update_users(&self, cli: &RtmClient) {
        let slack_users = KarmaManager::fetch_slack_user_list(cli);
        self.db_manager.update_users(&slack_users);
    }

    // Maybe rip this into the slack handler struct
    fn fetch_slack_user_list(cli: &RtmClient) -> Vec<UserRecord> {
        let slack_users: Vec<UserRecord> =
            cli.start_response().users.as_ref().unwrap().iter()
            .map(|user| {
                UserRecord {
                    id: None,
                    slack_id: user.id.clone(),
                    deleted: false,
                }
            }).collect();

        slack_users
    }
}

/*
 * Regular Expressions
 */
lazy_static! {
    // Slack now converts a mention into a user id of the form: <@U123>
    static ref INC_RE: Regex =
        Regex::new(r"<@([[:alpha:]0-9]+)>\+\+").unwrap();
    static ref DEC_RE: Regex =
        Regex::new(r"<@([[:alpha:]0-9]+)>--").unwrap();
}
