extern crate mysql;

use slack::{Message, Event, RtmClient, User};
use slack::Message::{Standard, MessageReplied};
use slack_api::{MessageStandard, MessageMessageReplied};
use regex::Regex;
use data_layer::DbManager;
use schema::{KarmaRecord, UserRecord};
use std::collections::HashMap;
use std::fmt;

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
            Standard(msg_std) => self.handle_std_msg(cli, &msg_std),
            MessageReplied(msg_reply) => self.handle_msg_reply(cli, &msg_reply),
            _ => return,
        }
    }

    pub fn handle_std_msg(&self, cli: &RtmClient, msg: &MessageStandard) {
        let text = &msg.text;
        let channel = &msg.channel.clone().unwrap();
        let s_id = &msg.user.clone().unwrap();

        match *text {
            Some(ref text) => self.handle_karma(cli, &text[..], channel, s_id),
            None => (),
        }
    }

    pub fn handle_msg_reply(&self, cli: &RtmClient, msg: &MessageMessageReplied) {
        let text = &msg.message.clone().unwrap().text;
        let channel = &msg.channel.clone().unwrap();
        let s_id = &msg.message.clone().unwrap().user.clone().unwrap();

        match *text {
            Some(ref text) => self.handle_karma(cli, &text[..], channel, s_id),
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

    // Handles and writes karma to the data store if there is any to distribute
    pub fn handle_karma(&self, cli: &RtmClient, text: &str,
        channel: &str, sender_id: &str) {
        let recipients = self.parse_recipients(text);

        match recipients {
            Some(ref recipients) if !recipients.is_empty() => {
                let db_users = self.db_manager.fetch_db_user_list();

                let mut records: Vec<KarmaRecord> = recipients.iter()
                    .filter(|ref tup| {
                        let mut is_recipient = false;

                        db_users.iter().for_each(|db_user| {
                            let slack_id = db_user.slack_id
                                .clone().unwrap_or("".to_string());

                            // Can't give karma to yourself
                            if slack_id != sender_id && slack_id == tup.0 {
                                is_recipient = true;
                                return;
                            }
                        });

                        is_recipient
                    })
                    .map(|ref tup| {
                        KarmaRecord {
                            id: None,
                            recipient: Some(tup.0.clone()),
                            donor: Some(sender_id.to_string()),
                            points: Some(tup.1),
                        }
                    }).collect();

                self.db_manager.write_karma(&records);

                // TODO make a DB call to get the SUM of each relevant
                // user to add to the response string. Add a response
                // string vector and pull a random string rom it to return.
                let mut response: String = String::new();
                for record in records.iter() {
                    response.push_str(format!("<@{}> karma [{}]\n",
                        record.recipient.clone().unwrap(),
                        record.points.clone().unwrap()).as_str());
                }

                cli.sender().send_message(channel, response.as_str()).unwrap();
            },
            Some(_) => (),
            None => (),
        }
    }

    // Parses each increment/decrement applied to a user and returns a
    // (String, i32) tuple representing (user, points).
    fn parse_recipients(&self, text: &str) -> Option<Vec<(String, i32)>> {
        let mut recipient_map = HashMap::new();

        for capture in INC_RE.captures_iter(text) {
            *recipient_map.entry(String::from(&capture[1])).or_insert(0) += 1;
        }

        for capture in DEC_RE.captures_iter(text) {
            *recipient_map.entry(String::from(&capture[1])).or_insert(0) += -1;
        }

        let recipient_vec: Vec<(String, i32)> = recipient_map.iter()
            .filter(|&(name, points)| *points != 0)
            .map(|(name, points)| (name.clone(), points.clone())).collect();

        match recipient_vec.is_empty() {
            true  => None,
            false => Some(recipient_vec),
        }
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
        Regex::new(r"<@([[:alpha:]0-9]+)>\s?\+\+").unwrap();
    static ref DEC_RE: Regex =
        Regex::new(r"<@([[:alpha:]0-9]+)>\s?--").unwrap();
}
