use chrono::{DateTime, NaiveDate, Utc};
use rusqlite;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

pub trait Sql {
    fn to_sql_insert(&self) -> String;
    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error>;
    fn to_json(&self) -> String;
    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error>;
}

#[derive(Serialize)]
pub struct JsonError {
    pub message: &'static str,
    pub code: usize,
    pub internal: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub root_path: String,
    pub bind_addr: String,
    pub bind_port: String,
    pub n_threads: usize,
    pub data_path: String,
}

#[derive(Clone)]
pub struct SessionUser {
    pub user_id: String,
    pub expire: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    id: Option<String>,
    assign_date: NaiveDate,
    due_date: NaiveDate,
    title: String,
    description: String,
    recurring_month: bool,
    recurring_n: bool,
    recurring_stop: String,
    //TODO consider how user_id should be handled
    //preferable not from client side
    #[serde(skip_serializing, skip_deserializing)]
    user_id: String,
}

impl Sql for Task {
    fn to_sql_insert(&self) -> String {
        format!("INSERT INTO tasks (id, assign_date, due_date, title, description, recurring_month, recurring_n, recurring_stop, user_id) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
            self.id.clone().unwrap_or(Uuid::now_v7().to_string()), 
            self.assign_date, 
            self.due_date,
            self.title,
            self.description,
            if self.recurring_month {1} else {0},
            if self.recurring_n {1} else {0},
            self.recurring_stop,
            self.user_id, )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error> {
        let t = Task {
            id: row.get("id")?,
            assign_date: row.get("assign_date")?,
            due_date: row.get("due_date")?,
            title: row.get("title")?,
            description: row.get("description")?,
            recurring_month: row.get("recurring_month")?,
            recurring_n: row.get("recurring_n")?,
            recurring_stop: row.get("recurring_stop")?,
            user_id: row.get("user_id")?,
        };
        Ok(Box::new(t))
    }

    fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error> {
        // let t: Task = serde_json::de::from_str(json)?;
        // Ok(Box::new(t))

        //Making a match here exposes the sillyness of this function
        match serde_json::de::from_str(json) {
            Ok(t) => Ok(Box::new(t)),
            Err(err) => Err(err),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub salt: Option<u8>,
}

impl Sql for User {
    fn to_sql_insert(&self) -> String {
        format!(
            "INSERT INTO users (id, username, password, salt) VALUES ('{}', '{}', '{}', {})",
            self.id.to_owned().unwrap_or(Uuid::now_v7().to_string()),
            self.username,
            self.password,
            self.salt.unwrap()
        )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error> {
        let u = User {
            id: row.get("id")?,
            username: row.get(1)?,
            password: row.get(2)?,
            salt: row.get(3)?,
        };
        Ok(Box::new(u))
    }

    fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error> {
        let u: User = serde_json::de::from_str(json)?;
        Ok(Box::new(u))
    }
}
