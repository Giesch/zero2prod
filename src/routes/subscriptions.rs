use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscribeReq {
    email: String,
    name: String,
}

// TODO remove this once sqlx releases the fix
// https://github.com/rust-lang/rust-clippy/issues/5849
#[allow(clippy::toplevel_ref_arg)]
pub async fn subscribe(
    form: web::Form<SubscribeReq>,
    connection: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Failed to execute query: {}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(HttpResponse::Ok().finish())
}
