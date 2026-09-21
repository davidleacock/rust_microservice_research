use crate::domain::{Task, TaskId};
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

    async fn update_task(&self, task: &Task) -> Result<(), RepositoryError> {
        let mut tasks = self
            .memory
            .lock()
            .map_err(|_| RepositoryError::GeneralError {
                message: "in-memory repo failed".to_string(),
            })?;

        if let Some(old_task) = tasks.get_mut(&task.task_id()) {
            *old_task = task.clone();
        }

        Ok(())
    }
}
