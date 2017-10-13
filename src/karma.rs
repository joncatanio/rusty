extern crate mysql;

use slack::RtmClient;
use slack::Message;
use slack::Message::Standard;
use regex::Regex;
use data_layer::DbManager;
use schema::{KarmaRecord, UserRecord};

pub struct KarmaManager {
    db_manager: DbManager,
}

impl KarmaManager {
    pub fn new() -> KarmaManager {
        println!("Initializing new `KarmaManager`...");

        KarmaManager { db_manager: DbManager::new() }
    }

    pub fn handle_message(&self, cli: &RtmClient, msg: Box<Message>) {
        println!("Handling message...");
        self.update_users(&cli);

        let text = match *msg {
            Standard(std_msg) => {
                println!("Sender: {}", std_msg.user.unwrap());
                std_msg.text
            },
            _ => None,
        };

        match text {
            Some(text) => self.parse_recipients(&text[..]),
            None => println!("not standard message"),
        }
    }

    fn parse_recipients(&self, text: &str) {
        let re = Regex::new(r"([[:alpha:]0-9]+)\+\+").unwrap();

        for capture in re.captures_iter(text) {
            println!("Captured: {} {}", &capture[0], &capture[1]);
        }
    }

    fn update_users(&self, cli: &RtmClient) {
        println!("Updating users...");
        let slack_users = KarmaManager::fetch_slack_user_list(cli);
        let db_users = self.db_manager.fetch_db_user_list();

        /*
        let new_users: Vec<&UserRecord> =
            slack_users.iter().filter(|slack_user| {
                db_users.iter().filter(|db_user| {
                    slack_user.slack_id.as_ref().unwrap()
                    == db_user.slack_id.as_ref().unwrap()
                }).collect::<Vec<&UserRecord>>().is_empty()
            }).collect();

        new_users.iter().for_each(|user| println!("NEW USER: {:?}", user));
        */

        self.db_manager.update_users(&slack_users);
        let records = vec![
            KarmaRecord {
                recipient: Some(String::from("U74JLD0MB")),
                donor: Some(String::from("U73S5T5MW")),
                points: Some(1),
            },
            KarmaRecord {
                recipient: Some(String::from("U75H354P9")),
                donor: Some(String::from("U75BBLRL6")),
                points: Some(1),
            },
            KarmaRecord {
                recipient: Some(String::from("U75H354P9")),
                donor: Some(String::from("U75BBLRL6")),
                points: Some(-1),
            },
        ];
        self.db_manager.write_karma(&records);
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

        for user in slack_users.iter() {
            println!("Slack User: {:?}", user);
        }

        slack_users
    }
}
