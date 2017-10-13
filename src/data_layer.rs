extern crate mysql;

use slack::RtmClient;
use schema::{KarmaRecord, UserRecord};

pub struct DbManager {
    pool: Option<mysql::Pool>,
}

impl DbManager {
    pub fn new() -> DbManager {
        println!("Initializing new `DbManager`...");

        let db_url = "mysql://rusty:rustykarmabot@localhost:3306/RustyKarma";
        DbManager { pool: Some(mysql::Pool::new(db_url).unwrap()) }
    }

    pub fn fetch_db_user_list(&self) -> Vec<UserRecord> {
		let db_users: Vec<UserRecord> =
			self.pool.as_ref().unwrap().prep_exec("SELECT * FROM Users", ())
			.map(|result| {
				result.map(|x| x.unwrap()).map(|row| {
					let (id, slack_id, nickname, first_name, last_name, email,
					    phone) = mysql::from_row(row);
					UserRecord {
						id,
						slack_id,
					    nickname,
					    first_name,
					    last_name,
					    email,
					    phone,
					}
				}).collect()
			}).unwrap();

		for user in db_users.iter() {
			println!("Database User: {:?}", user);
		}

		db_users
    }

    pub fn update_users(&self, users: &Vec<UserRecord>) {
        for mut stmt in self.pool.as_ref().unwrap().prepare("
            INSERT INTO Users
                (slack_id, nickname, first_name, last_name, email, phone)
            VALUES
                (:slack_id, :nickname, :first_name, :last_name, :email, :phone)
            ON DUPLICATE KEY UPDATE
                slack_id   = VALUES(slack_id),
                nickname   = VALUES(nickname),
                first_name = VALUES(first_name),
                last_name  = VALUES(last_name),
                email      = VALUES(email),
                phone      = VALUES(phone)
        ").into_iter() {
            for user in users.iter() {
                stmt.execute(params!{
                    "slack_id"   => user.slack_id.clone(),
                    "nickname"   => user.nickname.clone(),
                    "first_name" => user.first_name.clone(),
                    "last_name"  => user.last_name.clone(),
                    "email"      => user.email.clone(),
                    "phone"      => user.phone.clone(),
                }).unwrap();
            }
        }
    }
}
