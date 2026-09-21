use crate::domain::{Priority, ProjectId, Status, Task, TaskError, TaskId};
use crate::repository::TaskRepository;
use crate::task_service::{TaskService, TaskServiceError};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub service: TaskService,
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
    state: State<AppState>,
    Json(body): Json<CreateTask>,
) -> Result<Json<TaskId>, StatusCode> {
    let id = state
        .service
        .create_task(body.title, body.priority)
        .await
        .map_err(|e| match e {
            TaskServiceError::DomainError { .. } => StatusCode::BAD_REQUEST,
            TaskServiceError::RepositoryError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            TaskServiceError::GeneralError { .. } => StatusCode::NOT_FOUND,
        })?;

    Ok(Json(id))
}

pub async fn tasks(state: State<AppState>) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks: Vec<Task> = state
        .service
        .get_tasks()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tasks))
}

pub async fn get_task(
    state: State<AppState>,
    Path(task_id): Path<TaskId>,
) -> Result<Json<Task>, StatusCode> {
    let task = state.service.get_task(task_id).await.map_err(|e| match e {
        TaskServiceError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::NOT_FOUND,
    })?;

    Ok(Json(task))
}

pub async fn set_priority(
    state: State<AppState>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetPriority>,
) -> Result<Json<Task>, StatusCode> {
    let result = state
        .service
        .set_priority(task_id, body.priority)
        .await
        .map_err(|e| match e {
            TaskServiceError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::NOT_FOUND,
        })?;

    match result {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => Ok(Json(task)),
    }
}

pub async fn set_status(
    state: State<AppState>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetStatus>,
) -> Result<Json<Task>, StatusCode> {
    let result = state
        .service
        .set_status(task_id, body.status)
        .await
        .map_err(|e| match e {
            TaskServiceError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::NOT_FOUND,
        })?;

    match result {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => Ok(Json(task)),
    }
}

pub async fn set_project_id(
    state: State<AppState>,
    Path(task_id): Path<TaskId>,
    Json(body): Json<SetProjectId>,
) -> Result<Json<Task>, StatusCode> {
    let result = state
        .service
        .set_project_id(task_id, body.project_id)
        .await
        .map_err(|e| match e {
            TaskServiceError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::NOT_FOUND,
        })?;

    match result {
        None => Err(StatusCode::NOT_FOUND),
        Some(task) => Ok(Json(task)),
    }
}
