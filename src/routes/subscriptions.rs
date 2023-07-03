use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};
use tracing::Instrument;

#[derive(Deserialize)]
pub struct SubscriptionForm {
    pub email: String,
    pub name: String,
}

// If the `_form` cannot be parsed into `SubscriptionForm`
// then 400 (i.e.: `Bad Request`) is automatically returned.
pub async fn subscriptions(
    form: web::Form<SubscriptionForm>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
