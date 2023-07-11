pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

pub mod test_utils {
    use std::net::TcpListener;

    use once_cell::sync::Lazy;
    use secrecy::ExposeSecret;
    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use uuid::Uuid;

    use crate::{
        configuration::{get_configuration, DatabaseSettings},
        telemetry::{get_subscriber, init_subscriber},
    };

    static TRACING: Lazy<()> = Lazy::new(|| {
        let default_filter_level = "info";
        let subscriber_name = "test";

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(
                subscriber_name.into(),
                default_filter_level.into(),
                std::io::stdout,
            );
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(
                subscriber_name.into(),
                default_filter_level.into(),
                std::io::sink,
            );
            init_subscriber(subscriber);
        }
    });

    pub struct TestApp {
        pub address: String,
        pub db_pool: PgPool,
    }

    pub async fn spawn_app() -> TestApp {
        Lazy::force(&TRACING);
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
        let mut connection =
            PgConnection::connect(&config.connection_string_without_db().expose_secret())
                .await
                .expect("Failed to connect to Postgres.");

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create database.");

        let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate database.");

        connection_pool
    }
}
