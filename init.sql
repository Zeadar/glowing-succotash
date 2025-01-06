PRAGMA foreign_keys = ON;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    salt INTEGER NOT NULL
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    due_date TEXT NOT NULL,
    assign_date TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    recurring_month INTEGER NOT NULL,
    recurring_n INTEGER NOT NULL,
    recurring_stop TEXT NOT NULl,
    user_id TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE complete_tasks (
    id TEXT PRIMARY KEY,
    completed TEXT NOT NULL,
    task_id TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE TABLE skip_tasks (
    id TEXT PRIMARY KEY,
    completed TEXT NOT NULL,
    task_id TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE TABLE subtasks (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    task_id TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE TABLE skip_subtasks (
    id TEXT PRIMARY KEY,
    completed TEXT NOT NULL,
    subtask_id TEXT NOT NULL,
    FOREIGN KEY (subtask_id) REFERENCES subtasks(id)
);
    
CREATE TABLE complete_subtasks (
    id TEXT PRIMARY KEY,
    completed TEXT NOT NULL,
    subtask_id TEXT NOT NULL,
    FOREIGN KEY (subtask_id) REFERENCES subtasks(id)
);
