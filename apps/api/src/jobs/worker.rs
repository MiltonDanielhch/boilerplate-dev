// Ubicación: `apps/api/src/jobs/worker.rs`
//
// Descripción: Configuración de workers de Apalis.
//
// ADRs relacionados: ADR 0018

use apalis::prelude::*;
use apalis_sql::SqliteStorage;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::cleanup_job::{CleanupJob, CleanupTask};
use super::email_job::{EmailJob, EmailJobHandler};

pub async fn create_email_storage(pool: SqlitePool) -> SqliteStorage<EmailJob> {
    let storage = SqliteStorage::new(pool);
    storage
        .run()
        .await
        .expect("Failed to initialize email job storage");
    storage
}

pub async fn create_cleanup_storage(pool: SqlitePool) -> SqliteStorage<CleanupJob> {
    let storage = SqliteStorage::new(pool);
    storage
        .run()
        .await
        .expect("Failed to initialize cleanup job storage");
    storage
}

pub async fn start_workers(
    pool: SqlitePool,
    mailer: Box<dyn crate::mailer::ports::MailerPort>,
) {
    let pool = Arc::new(pool);
    let pool_for_email = pool.clone();
    
    let email_storage = create_email_storage((*pool_for_email).clone()).await;
    let cleanup_storage = create_cleanup_storage((*pool).clone()).await;

    let email_handler = EmailJobHandler::new(mailer);

    tokio::spawn(async move {
        let mut monitor = Monitor::new();
        
        let email_worker = WorkerBuilder::new(email_storage)
            .layer(apalis::layers::TraceLayer::new())
            .layer(apalis::layers::RetryLayer::new(
                apalis::layers::RetryPolicy::retries(3),
            ))
            .build(email_handler);
            
        let cleanup_worker = WorkerBuilder::new(cleanup_storage)
            .layer(apalis::layers::TraceLayer::new())
            .layer(apalis::layers::RetryLayer::new(
                apalis::layers::RetryPolicy::retries(3),
            ))
            .build_fn(move |job: CleanupJob| {
                let pool = pool.clone();
                async move {
                    job.run(JobContext::new().await).await
                }
            });

        monitor.register(email_worker);
        monitor.register(cleanup_worker);

        tracing::info!("Starting background workers");
        monitor.run().await;
    });
}

pub async fn enqueue_email(
    storage: &SqliteStorage<EmailJob>,
    job: EmailJob,
) -> Result<(), apalis::prelude::JobError> {
    storage
        .push(job)
        .await
        .map_err(|e| apalis::prelude::JobError::Unexpected(e.to_string()))
}

pub async fn enqueue_cleanup(
    storage: &SqliteStorage<CleanupJob>,
    pool: Arc<SqlitePool>,
    task: CleanupTask,
) -> Result<(), apalis::prelude::JobError> {
    let job = CleanupJob::new(task, pool);
    storage
        .push(job)
        .await
        .map_err(|e| apalis::prelude::JobError::Unexpected(e.to_string()))
}