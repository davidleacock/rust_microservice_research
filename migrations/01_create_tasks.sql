CREATE TABLE IF NOT EXISTS tasks (
    task_id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    project_id UUID
);