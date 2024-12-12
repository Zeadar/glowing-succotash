use chrono::NaiveDate;
use rusqlite;
use serde::{Deserialize, Serialize};
use serde_json;

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
    id: Option<u64>,
    assign_date: NaiveDate,
    due_date: NaiveDate,
    title: String,
    description: String,
}

impl Sql for Task {
    type Me = Self;

    fn to_sql_insert(&self) -> String {
        format!("INSERT INTO tasks (assign_date, due_date, title, description) VALUES ('{}', '{}', '{}', '{}');",
            self.assign_date, self.due_date, self.title, self.description)
    }

    fn from_sql_row(row: &rusqlite::Row) -> Result<Self::Me, rusqlite::Error> {
        let t = Task {
            id: row.get(0)?,
            assign_date: row.get(2)?,
            due_date: row.get(1)?,
            title: row.get(3)?,
            description: row.get(4)?,
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
