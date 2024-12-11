CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    due_date TEXT NOT NULL,
    assign_date TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
