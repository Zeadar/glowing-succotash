CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    duedate NUMERIC NOT NULL,
    assigndate NUMERIC NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
