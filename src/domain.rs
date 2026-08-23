use Status::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TaskId(pub Uuid);
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct ProjectId(pub Uuid);

#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("invalid status transition: {message}")]
    InvalidTransition { message: String },
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl Status {
    pub fn transition_to(&self, status: Status) -> Result<Status, StatusError> {
        match self {
            Todo => {
                if matches!(status, Done) {
                    return Err(StatusError::InvalidTransition {
                        message: "Todo cannot be moved to Done, must either Cancel or InProgress"
                            .to_string(),
                    });
                }
                Ok(status)
            }
            InProgress => Ok(status),
            Done => Err(StatusError::InvalidTransition {
                message: "Issue is in Done, cannot move.".to_string(),
            }),
            Cancelled => Err(StatusError::InvalidTransition {
                message: "Issue is Cancelled, cannot move.".to_string(),
            }),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    task_id: TaskId,
    title: String,
    priority: Priority,
    status: Status,
    project_id: Option<ProjectId>,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Must provide title for Task.")]
    TitleMissing,
    #[error("Unable to change status: {message}")]
    InvalidStatusChange { message: String },
}

impl Task {
    pub fn new(title: String, priority: Priority) -> Result<Task, TaskError> {
        if title.is_empty() {
            return Err(TaskError::TitleMissing);
        }

        let status = Todo;
        let task_id = TaskId(Uuid::new_v4());

        Ok(Task {
            task_id,
            title,
            priority,
            status,
            project_id: None,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn project_id(&self) -> Option<ProjectId> {
        self.project_id
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    pub fn set_project_id(&mut self, project_id: ProjectId) {
        self.project_id = Some(project_id)
    }

    pub fn set_status(&mut self, status: Status) -> Result<(), TaskError> {
        match self.status.transition_to(status) {
            Ok(new_status) => {
                self.status = new_status;
                Ok(())
            }
            Err(error) => match error {
                StatusError::InvalidTransition { message } => {
                    Err(TaskError::InvalidStatusChange { message })
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{Priority, Status, StatusError, Task, TaskError};
    use Priority::*;
    use Status::*;

    #[test]
    fn validate_new_task() {
        let task = Task::new("test".to_string(), Low).expect("failed to create task");

        assert!(matches!(
            task,
            Task {
                project_id: None,
                status: Todo,
                ..
            }
        ));
    }

    #[test]
    fn task_requires_a_title() {
        let task = Task::new("".to_string(), Low);

        assert!(matches!(task, Err(TaskError::TitleMissing)))
    }

    #[test]
    fn validate_set_status_in_task() {
        let mut task = Task::new("test".to_string(), Low).expect("failed to create task");

        assert!(matches!(task.set_status(InProgress), Ok(())))
    }

    #[test]
    fn validate_set_status_in_task_invalid_transition() {
        let mut task = Task::new("test".to_string(), Low).expect("failed to create task");

        assert!(matches!(
            task.set_status(Done),
            Err(TaskError::InvalidStatusChange { message: _ })
        ))
    }

    #[test]
    fn validate_transition_status() {
        // _Todo
        assert!(matches!(Todo.transition_to(InProgress), Ok(InProgress)));

        assert!(matches!(
            Todo.transition_to(Done),
            Err(StatusError::InvalidTransition { message }) if message == "Todo cannot be moved to Done, must either Cancel or InProgress"
        ));

        assert!(matches!(Todo.transition_to(Cancelled), Ok(Cancelled)));

        // InProgress
        assert!(matches!(InProgress.transition_to(Done), Ok(Done)));

        assert!(matches!(InProgress.transition_to(Todo), Ok(Todo)));

        assert!(matches!(InProgress.transition_to(Cancelled), Ok(Cancelled)));

        // Done
        assert!(matches!(
            Done.transition_to(Todo),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is in Done, cannot move."
        ));

        assert!(matches!(
            Done.transition_to(InProgress),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is in Done, cannot move."
        ));

        assert!(matches!(
            Done.transition_to(Cancelled),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is in Done, cannot move."
        ));

        // Cancelled
        assert!(matches!(
            Cancelled.transition_to(Todo),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is Cancelled, cannot move."
        ));

        assert!(matches!(
            Cancelled.transition_to(InProgress),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is Cancelled, cannot move."
        ));

        assert!(matches!(
            Cancelled.transition_to(Cancelled),
            Err(StatusError::InvalidTransition { message }) if message == "Issue is Cancelled, cannot move."
        ));
    }
}
