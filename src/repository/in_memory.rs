use crate::domain::{Priority, ProjectId, Status, Task, TaskId};
use crate::repository::{RepositoryError, TaskRepository};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct InMemoryTaskRepository {
    pub memory: Mutex<HashMap<TaskId, Task>>,
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn get_task(&self, task_id: TaskId) -> Result<Option<Task>, RepositoryError> {
        let tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        match tasks.get(&task_id) {
            None => Ok(None),
            Some(task) => Ok(Some(task.clone())),
        }
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, RepositoryError> {
        let tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        let results: Vec<Task> = tasks.values().cloned().collect();

        Ok(results)
    }

    async fn create_task(&self, task: Task) -> Result<TaskId, RepositoryError> {
        let mut tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        let id = task.task_id();
        tasks.insert(task.task_id(), task);

        Ok(id)
    }

    async fn set_priority(
        &self,
        task_id: TaskId,
        priority: Priority,
    ) -> Result<Option<Task>, RepositoryError> {
        let mut tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        Ok(if let Some(task) = tasks.get_mut(&task_id) {
            task.set_priority(priority);
            Some(task.clone())
        } else {
            None
        })
    }

    async fn set_status(
        &self,
        task_id: TaskId,
        status: Status,
    ) -> Result<Option<Task>, RepositoryError> {
        let mut tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        Ok(if let Some(task) = tasks.get_mut(&task_id) {
            task.set_status(status)
                .map_err(|_| RepositoryError::GeneralError {
                    message: "in-memory repo failed".to_string(),
                })?;
            Some(task.clone())
        } else {
            None
        })
    }

    async fn set_project_id(
        &self,
        task_id: TaskId,
        project_id: ProjectId,
    ) -> Result<Option<Task>, RepositoryError> {
        let mut tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        Ok(if let Some(task) = tasks.get_mut(&task_id) {
            task.set_project_id(project_id);
            Some(task.clone())
        } else {
            None
        })
    }
}
