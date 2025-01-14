use chrono::{DateTime, NaiveDate, Utc};
use rand::random;
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

#[derive(Deserialize, Serialize)]
pub struct IdCarrier {
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    #[serde(skip_deserializing)]
    id: String,
    #[serde(rename = "assignDate")]
    assign_date: NaiveDate,
    title: String,
    description: String,
    #[serde(rename = "recurringMonth")]
    recurring_month: bool,
    #[serde(rename = "recurringN")]
    recurring_n: u32,
    #[serde(rename = "recurringStop")]
    recurring_stop: NaiveDate,
}

impl Sql for Task {
    fn to_sql_insert(&self) -> String {
        format!("INSERT INTO tasks (id, assign_date, title, description, recurring_month, recurring_n, recurring_stop, user_id) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{{}}');",
            self.id,
            self.assign_date,
            self.title,
            self.description,
            if self.recurring_month {1} else {0},
            self.recurring_n,
            self.recurring_stop,
       )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error> {
        let t = Self {
            id: row.get("id")?,
            assign_date: row.get("assign_date")?,
            title: row.get("title")?,
            description: row.get("description")?,
            recurring_month: row.get("recurring_month")?,
            recurring_n: row.get("recurring_n")?,
            recurring_stop: row.get("recurring_stop")?,
        };
        Ok(Box::new(t))
    }

    fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error> {
        let mut t: Task = serde_json::de::from_str(json)?;
        t.id = Uuid::now_v7().to_string();
        Ok(Box::new(t))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompleteTask {
    #[serde(skip_deserializing)]
    id: String,
    completed: NaiveDate,
    #[serde(skip_serializing)]
    task_id: String,
}

impl Sql for CompleteTask {
    fn to_sql_insert(&self) -> String {
        format!(
            "INSERT INTO complete_tasks (id, completed, task_id) VALUES ('{}', '{}', '{}');",
            self.id, self.completed, self.task_id,
        )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error> {
        let ct = Self {
            id: row.get("id")?,
            completed: row.get("completed")?,
            task_id: row.get("task_id")?,
        };
        Ok(Box::new(ct))
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error> {
        let mut ct = serde_json::from_str::<CompleteTask>(json)?;
        ct.id = Uuid::now_v7().to_string();
        Ok(Box::new(ct))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(skip_deserializing)]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub salt: u8,
}

impl Sql for User {
    fn to_sql_insert(&self) -> String {
        format!(
            "INSERT INTO users (id, username, password, salt) VALUES ('{}', '{}', '{}', {})",
            self.id, self.username, self.password, self.salt
        )
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Box<Self>, rusqlite::Error> {
        let u = User {
            id: row.get("id")?,
            username: row.get("username")?,
            password: row.get("password")?,
            salt: row.get("salt")?,
        };
        Ok(Box::new(u))
    }

    fn to_json(&self) -> String {
        serde_json::ser::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Result<Box<Self>, serde_json::Error> {
        let mut user: User = serde_json::de::from_str(json)?;
        user.id = Uuid::now_v7().to_string();
        user.salt = random();
        Ok(Box::new(user))
    }
}
