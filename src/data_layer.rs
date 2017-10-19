extern crate mysql;

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
			self.pool.as_ref().unwrap().prep_exec("
			    SELECT id, slack_id, deleted FROM Users
			", ()).map(|result| {
				result.map(|x| x.unwrap()).map(|row| {
					let (id, slack_id, deleted) = mysql::from_row(row);
					UserRecord {
						id,
						slack_id,
					    deleted,
					}
				}).collect()
			}).unwrap();
		db_users
    }

    // TODO make sure to delete users that are no longer in the slack group
    pub fn update_users(&self, records: &Vec<UserRecord>) {
        for mut stmt in self.pool.as_ref().unwrap().prepare("
            INSERT INTO Users
                (slack_id, deleted, created)
            VALUES
                (:slack_id, :deleted, NOW())
            ON DUPLICATE KEY UPDATE
                slack_id = VALUES(slack_id),
                deleted  = VALUES(deleted)
        ").into_iter() {
            for record in records.iter() {
                stmt.execute(params!{
                    "slack_id" => record.slack_id.clone(),
                    "deleted"  => record.deleted.clone(),
                }).unwrap();
            }
        }
    }

    // The database users list should be updated before making calls to
    // `write_karma`, it will silently update the table with no records if
    // it can't find a recipient or donor `slack_id`.
    pub fn write_karma(&self, records: &Vec<KarmaRecord>) {
        for mut stmt in self.pool.as_ref().unwrap().prepare("
            INSERT INTO Karma
                (recipient, donor, points, created)
            SELECT R.id, D.id, :points, NOW()
            FROM
                (SELECT id FROM Users WHERE slack_id = :r_slack_id) as R
                JOIN (SELECT id FROM Users WHERE slack_id = :d_slack_id) as D
        ").into_iter() {
            for record in records.iter() {
                stmt.execute(params!{
                    "points"     => record.points.clone(),
                    "r_slack_id" => record.recipient.clone(),
                    "d_slack_id" => record.donor.clone(),
                }).unwrap();
            }
        }
    }

    // TODO implement to get the karma SUM of each user. If `users` is None
    // return karma for all users.
    pub fn get_karma(&self, users: Option<&Vec<String>>) {
        // unimplemented
    }
}
