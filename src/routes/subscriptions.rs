use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

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
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscriptions(
    form: web::Form<SubscriptionForm>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let (name, email) = match (
        SubscriberName::parse(form.0.name),
        SubscriberEmail::parse(form.0.email),
    ) {
        (Ok(name), Ok(email)) => (name, email),
        _ => return HttpResponse::BadRequest().finish(),
    };

    let new_subscriber = NewSubscriber { email, name };
    match insert_subscriber(&new_subscriber, &db).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber in the database",
    skip(new_subscriber, db)
)]
async fn insert_subscriber(new_subscriber: &NewSubscriber, db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
