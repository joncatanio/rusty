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

        match recipients {
            Some(ref recipients) if !recipients.is_empty() => {
                let db_users = self.db_manager.fetch_db_user_list();
                let mut records: Vec<&UserRecord> = Vec::new();

                records = db_users.iter().filter(|db_user| {
                    let mut recipient = false;

                    recipients.iter().for_each(|ref tup| {
                        let nickname = db_user.nickname
                            .clone().unwrap_or("".to_string()).to_lowercase();
                        let first_name = db_user.first_name
                            .clone().unwrap_or("".to_string()).to_lowercase();
                        let last_name = db_user.last_name
                            .clone().unwrap_or("".to_string()).to_lowercase();

                        if nickname == tup.0.to_lowercase()
                            || first_name == tup.0.to_lowercase()
                            || last_name == tup.0.to_lowercase() {
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
        let profile = user.profile.as_ref().unwrap();
        let user_vec = vec![
            UserRecord {
                id: None,
                slack_id: user.id.clone(),
                nickname: user.name.clone(),
                first_name: profile.first_name.clone(),
                last_name: profile.last_name.clone(),
                email: profile.email.clone(),
                phone: profile.phone.clone(),
                deleted: false,
            },
        ];

        println!("Updating User: {:?}", user_vec);

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
                let profile = user.profile.as_ref().unwrap();

                UserRecord {
                    id: None,
                    slack_id: user.id.clone(),
                    nickname: user.name.clone(),
                    first_name: profile.first_name.clone(),
                    last_name: profile.last_name.clone(),
                    email: profile.email.clone(),
                    phone: profile.phone.clone(),
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
    static ref INC_RE: Regex = Regex::new(r"([[:alpha:]0-9]+)\+\+").unwrap();
    static ref DEC_RE: Regex = Regex::new(r"([[:alpha:]0-9]+)--").unwrap();
}
