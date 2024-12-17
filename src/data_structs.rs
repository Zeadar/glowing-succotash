use chrono::NaiveDate;
use rand;
use rusqlite;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

pub trait Sql {
    type Me;

    fn to_sql_insert(&self) -> String;
    fn from_sql_row(row: &rusqlite::Row) -> Result<Self::Me, rusqlite::Error>;
    fn to_json(&self) -> String;
    fn from_json(json: &str) -> Result<Self::Me, serde_json::Error>;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub root_path: String,
    pub bind_addr: String,
    pub bind_port: String,
    pub n_threads: usize,
    pub data_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    id: Option<String>,
    assign_date: NaiveDate,
    due_date: NaiveDate,
    title: String,
    description: String,
    #[serde(skip_serializing)]
    user_id: String,
}

impl Sql for Task {
    type Me = Self;

    fn to_sql_insert(&self) -> String {
        format!("INSERT INTO tasks (id, assign_date, due_date, title, description, user_id) VALUES ('{}', '{}', '{}', '{}', '{}', '{}');",
            self.id.clone().unwrap_or(Uuid::now_v7().to_string()), self.assign_date, self.due_date, self.title, self.description, self.user_id )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Self::Me, rusqlite::Error> {
        let t = Task {
            id: row.get(0)?,
            assign_date: row.get(2)?,
            due_date: row.get(1)?,
            title: row.get(3)?,
            description: row.get(4)?,
            user_id: row.get(5)?,
        };
        Ok(t)
    }

    fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Self::Me, serde_json::Error> {
        let t: Task = serde_json::de::from_str(json)?;
        Ok(t)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    id: Option<String>,
    username: String,
    #[serde(skip_serializing)]
    password: String,
    #[serde(skip_serializing)]
    salt: Option<u64>,
}

impl Sql for User {
    type Me = Self;

    fn to_sql_insert(&self) -> String {
        format!(
            "INSERT INTO users (id, username, password, salt) VALUES ('{}', '{}', '{}', {})",
            self.id.to_owned().unwrap_or(Uuid::now_v7().to_string()),
            self.username,
            self.password,
            self.salt.to_owned().unwrap_or(rand::random()),
        )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Self::Me, rusqlite::Error> {
        let u = User {
            id: row.get("id")?,
            username: row.get(1)?,
            password: row.get(2)?,
            salt: row.get(3)?,
        };
        Ok(u)
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Self::Me, serde_json::Error> {
        let u: User = serde_json::de::from_str(json)?;
        Ok(u)
    }
}
