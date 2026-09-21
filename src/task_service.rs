use crate::domain::{Priority, ProjectId, Status, Task, TaskId};
use crate::repository::TaskRepository;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum TaskServiceError {
    #[error("Invalid Task operation: {0}")]
    DomainError(#[from] crate::domain::TaskError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] crate::repository::RepositoryError),
    #[error("Error: {message}")]
    GeneralError { message: String },
}

#[derive(Clone)]
pub struct TaskService {
    pub repository: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub async fn create_task(
        &self,
        title: String,
        priority: Priority,
    ) -> Result<TaskId, TaskServiceError> {
        let task = Task::new(title, priority)?;
        let id = self.repository.create_task(task).await?;

        Ok(id)
    }

    pub async fn get_task(&self, task_id: TaskId) -> Result<Task, TaskServiceError> {
        self.repository
            .get_task(task_id)
            .await
            .map(|maybe_task| match maybe_task {
                None => Err(TaskServiceError::GeneralError {
                    message: "Task not found".to_string(),
                }),
                Some(task) => Ok(task),
            })?
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, TaskServiceError> {
        let tasks = self.repository.get_tasks().await?;

        Ok(tasks)
    }

    pub async fn set_priority(
        &self,
        task_id: TaskId,
        priority: Priority,
    ) -> Result<Option<Task>, TaskServiceError> {
        let maybe_task = self.repository.get_task(task_id).await?;

        match maybe_task {
            None => Ok(None),
            Some(mut task) => {
                task.set_priority(priority);
                self.repository.update_task(&task).await?;
                Ok(Some(task))
            }
        }
    }

    pub async fn set_status(
        &self,
        task_id: TaskId,
        status: Status,
    ) -> Result<Option<Task>, TaskServiceError> {
        let maybe_task = self.repository.get_task(task_id).await?;

        match maybe_task {
            None => Ok(None),
            Some(mut task) => {
                task.set_status(status)?;
                self.repository.update_task(&task).await?;
                Ok(Some(task))
            }
        }
    }

    pub async fn set_project_id(
        &self,
        task_id: TaskId,
        project_id: ProjectId,
    ) -> Result<Option<Task>, TaskServiceError> {
        let maybe_task = self.repository.get_task(task_id).await?;

        match maybe_task {
            None => Ok(None),
            Some(mut task) => {
                task.set_project_id(project_id);
                self.repository.update_task(&task).await?;
                Ok(Some(task))
            }
        }
    }
}
