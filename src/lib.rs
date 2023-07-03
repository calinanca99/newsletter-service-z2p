pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub mod test_utils {
    use std::net::TcpListener;

    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use uuid::Uuid;

    use crate::configuration::{get_configuration, DatabaseSettings};

    pub struct TestApp {
        pub address: String,
        pub db_pool: PgPool,
    }

    pub async fn spawn_app() -> TestApp {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let mut configuration = get_configuration().expect("Failed to read configuration.");
        configuration.database.database_name = Uuid::new_v4().to_string();

        let connection_pool = configure_database(&configuration.database).await;

        let server = crate::startup::run(listener, connection_pool.clone())
            .expect("Failed to bind to address");

        tokio::spawn(server);

        TestApp {
            address,
            db_pool: connection_pool,
        }
    }

    async fn configure_database(config: &DatabaseSettings) -> PgPool {
        let mut connection = PgConnection::connect(&config.connection_string_without_db())
            .await
            .expect("Failed to connect to Postgres.");

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create database.");

        let connection_pool = PgPool::connect(&config.connection_string())
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate database.");

        connection_pool
    }
}
