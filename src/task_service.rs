use crate::domain::{Priority, ProjectId, Status, Task, TaskId};
use crate::repository::TaskRepository;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum TaskServiceError {
    #[error("Invalid Task operation: {message}")]
    DomainError { message: String },
    #[error("Repository error: {message}")]
    RepositoryError { message: String },
}

#[derive(Clone)]
pub struct TaskService {
    pub repository: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn create_task(&self, task: Task) -> Result<TaskId, TaskServiceError> {
        todo!()
    }

    pub async fn get_task(&self, task_id: TaskId) -> Result<Option<Task>, TaskServiceError> {
        todo!()
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, TaskServiceError> {
        todo!()
    }

    pub async fn set_priority(
        &self,
        task_id: TaskId,
        priority: Priority,
    ) -> Result<Option<Task>, TaskServiceError> {
        todo!()
    }

    pub async fn set_status(
        &self,
        task_id: TaskId,
        status: Status,
    ) -> Result<Option<Task>, TaskServiceError> {
        todo!()
    }

    pub async fn set_project_id(
        &self,
        task_id: TaskId,
        project_id: ProjectId,
    ) -> Result<Option<Task>, TaskServiceError> {
        todo!()
    }
}
