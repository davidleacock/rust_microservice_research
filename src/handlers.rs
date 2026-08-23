use crate::domain::{Priority, ProjectId, Status, Task, TaskError, TaskId};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub tasks: Mutex<HashMap<TaskId, Task>>,
}

#[derive(Deserialize)]
pub struct CreateTask {
    title: String,
    priority: Priority,
}

#[derive(Deserialize)]
pub struct SetPriority {
    priority: Priority,
}

#[derive(Deserialize)]
pub struct SetStatus {
    status: Status,
}

#[derive(Deserialize)]
pub struct SetProjectId {
    project_id: ProjectId,
}

pub async fn create_task(
    state: State<Arc<AppState>>,
    Json(body): Json<CreateTask>,
) -> Result<Json<TaskId>, StatusCode> {
    match Task::new(body.title, body.priority) {
        Ok(task) => {
            let id = task.task_id();
            let mut tasks = state
                .tasks
                .lock()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            tasks.insert(task.task_id(), task);
            Ok(Json(id))
        }
        Err(error) => match error {
            TaskError::TitleMissing => Err(StatusCode::BAD_REQUEST),
            TaskError::InvalidStatusChange { .. } => Err(StatusCode::METHOD_NOT_ALLOWED),
        },
    }
}

pub async fn tasks(state: State<Arc<AppState>>) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tasks: Vec<Task> = tasks.values().cloned().collect();
    Ok(Json(tasks))
}

pub async fn get_task(
    state: State<Arc<AppState>>,
    Path(task_id): Path<TaskId>,
) -> Result<Json<Task>, StatusCode> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match tasks.get(&task_id) {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => Ok(Json(task.clone())),
    }
}

pub async fn set_priority(
    state: State<Arc<AppState>>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetPriority>,
) -> Result<Json<Task>, StatusCode> {
    let mut tasks = state
        .tasks
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match tasks.get_mut(&task_id) {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => {
            task.set_priority(body.priority);
            Ok(Json(task.clone()))
        }
    }
}

pub async fn set_status(
    state: State<Arc<AppState>>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetStatus>,
) -> Result<Json<Task>, StatusCode> {
    let mut tasks = state
        .tasks
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match tasks.get_mut(&task_id) {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => match task.set_status(body.status) {
            Ok(_) => Ok(Json(task.clone())),
            Err(message) => match message {
                TaskError::InvalidStatusChange { .. } => Err(StatusCode::CONFLICT),
                _ => Err(StatusCode::NOT_FOUND),
            },
        },
    }
}

pub async fn set_project_id(
    state: State<Arc<AppState>>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetProjectId>,
) -> Result<(), StatusCode> {
    let mut tasks = state
        .tasks
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match tasks.get_mut(&task_id) {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => {
            task.set_project_id(body.project_id);
            Ok(())
        }
    }
}
