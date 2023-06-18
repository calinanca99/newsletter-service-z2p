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
pub async fn subscriptions(
    form: web::Form<SubscriptionForm>,
    connection: web::Data<PgPool>,
) -> impl Responder {
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
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
