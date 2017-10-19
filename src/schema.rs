// MySQL schema definitions
#[derive(Debug)]
pub struct KarmaRecord {
    pub id: Option<i32>,
    pub recipient: Option<String>,
    pub donor: Option<String>,
    pub points: Option<i32>,
}

#[derive(Debug)]
pub struct UserRecord {
    pub id: Option<i32>,
    pub slack_id: Option<String>,
    pub deleted: bool,
}
