use std::char::CharTryFromError;

use chrono::NaiveDate;
use rusqlite;
use serde::{Deserialize, Serialize};

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
    assign_date: NaiveDate,
    due_date: NaiveDate,
    title: String,
    description: String,
}

impl Task {
    pub fn to_sql(&self) -> String {
        format!("INSERT INTO tasks (assign_date, due_date, title, description) VALUES ('{}', '{}', '{}'. '{}')", self.assign_date, self.due_date, self.title, self.description)
    }

    //TODO implement error
    // pub fn from_sql_row(row: &rusqlite::Row) -> Result<(), CharTryFromError>
}
