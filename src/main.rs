use axum::routing::post;
use axum::{Router, routing::get};
use std::sync::{Arc, Mutex};
use taskflow::handlers::{
    AppState, create_task, get_task, set_priority, set_project_id, set_status, tasks,
};
use taskflow::repository::InMemoryTaskRepository;
use taskflow::task_service::TaskService;

#[tokio::main]
async fn main() {
    let state = AppState {
        task_service: TaskService {
            repository: Arc::new(InMemoryTaskRepository {
                memory: Mutex::new(Default::default()),
            }),
        },
    };

    let app = Router::new()
        .route("/", get(|| async { "server running..." }))
        .route("/task", post(create_task))
        .route("/tasks", get(tasks))
        .route("/task/{task_id}", get(get_task))
        .route("/task/{task_id}/priority", post(set_priority))
        .route("/task/{task_id}/status", post(set_status))
        .route("/task/{task_id}/project_id", post(set_project_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}
