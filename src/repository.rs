mod in_memory;
mod postgres;

use crate::domain::{Priority, ProjectId, Status, Task, TaskId};
use async_trait::async_trait;

pub use in_memory::InMemoryTaskRepository;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("invalid status transition: {message}")]
    GeneralError { message: String },
}

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn get_task(&self, task_id: TaskId) -> Result<Option<Task>, RepositoryError>;
    async fn get_tasks(&self) -> Result<Vec<Task>, RepositoryError>;
    async fn create_task(&self, task: Task) -> Result<TaskId, RepositoryError>;
    async fn update_task(&self, task: &Task) -> Result<(), RepositoryError>;
}
