use crate::domain::{Task, TaskId};
use crate::repository::{RepositoryError, TaskRepository};
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use sqlx::{query, query_as};
use uuid::Uuid;

pub struct PostgresTaskRepository {
    pub pool: PgPool,
}

#[derive(sqlx::FromRow)]
struct TaskDto {
    task_id: Uuid,
    title: String,
    priority: String,
    status: String,
    project_id: Option<Uuid>,
}

#[async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn get_task(&self, task_id: TaskId) -> Result<Option<Task>, RepositoryError> {
        let query = query_as!(TaskDto, "SELECT * FROM tasks WHERE task_id = $1", task_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::GeneralError {
                message: e.to_string(),
            })?;

        match query {
            None => Ok(None),
            Some(row) => {
                let task = Task::from_dto(
                    row.task_id,
                    row.title,
                    row.priority,
                    row.status,
                    row.project_id,
                );

                Ok(Some(task))
            }
        }
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, RepositoryError> {
        let tasks = query_as!(TaskDto, "SELECT * FROM tasks")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::GeneralError {
                message: e.to_string(),
            })
            .map(|query_result| {
                query_result
                    .into_iter()
                    .map(|dto| {
                        Task::from_dto(
                            dto.task_id,
                            dto.title,
                            dto.priority,
                            dto.status,
                            dto.project_id,
                        )
                    })
                    .collect()
            })?;

        Ok(tasks)
    }

    async fn create_task(&self, task: Task) -> Result<TaskId, RepositoryError> {
        query(
            "INSERT INTO tasks (task_id, title, priority, status, project_id) VALUES ( $1, $2, $3, $4, $5)"
        )
            .bind(task.task_id().0)
            .bind(task.title())
            .bind(task.priority().to_string())
            .bind(task.status().to_string())
            .bind(task.project_id().map(|id| id.0))
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::GeneralError {
                message: e.to_string(),
            })?;

        Ok(task.task_id())
    }

    async fn update_task(&self, task: &Task) -> Result<(), RepositoryError> {
        query(
            r#"
        UPDATE tasks
        SET title = $1, priority = $2, status = $3, project_id = $4
        WHERE task_id = $5"#,
        )
        .bind(task.title())
        .bind(task.priority().to_string())
        .bind(task.status().to_string())
        .bind(task.project_id().map(|id| id.0))
        .bind(task.task_id().0)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::GeneralError {
            message: e.to_string(),
        })?;

        Ok(())
    }
}
