use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

#[derive(Deserialize)]
pub struct SubscriptionForm {
    pub email: String,
    pub name: String,
}

// If the `_form` cannot be parsed into `SubscriptionForm`
// then 400 (i.e.: `Bad Request`) is automatically returned.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscriptions(
    form: web::Form<SubscriptionForm>,
    db: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscriber(&form, &db).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving new subscriber in the database", skip(form, db))]
async fn insert_subscriber(form: &SubscriptionForm, db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
