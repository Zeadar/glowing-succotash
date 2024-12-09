use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub root_path: String,
    pub bind_addr: String,
    pub bind_port: String,
    pub n_threads: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
    assign_date: NaiveDate,
    due_date: NaiveDate,
    title: String,
    description: String,
}
