use axum::routing::post;
use axum::{Router, routing::get};
use sqlx::PgPool;
use std::sync::Arc;
use taskflow::handlers::{
    AppState, create_task, get_task, set_priority, set_project_id, set_status, tasks,
};
use taskflow::repository::PostgresTaskRepository;
use taskflow::task_service::TaskService;

#[tokio::main]
async fn main() {
    let conn_url =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = PgPool::connect(&conn_url)
        .await
        .expect("unable to connect to postgres");

    let state = AppState {
        task_service: TaskService {
            repository: Arc::new(PostgresTaskRepository { pool }),
        },
    };

    // let state = AppState {
    //     task_service: TaskService {
    //         repository: Arc::new(InMemoryTaskRepository {
    //             memory: Mutex::new(Default::default()),
    //         }),
    //     },
    // };

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
