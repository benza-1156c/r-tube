use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};

use crate::db::entities::profile;
use crate::error::AppError;

pub async fn get_active_profile(
    db: &DatabaseConnection,
) -> Result<Option<profile::Model>, AppError> {
    let profile = profile::Entity::find()
        .filter(profile::Column::IsActive.eq(true))
        .one(db)
        .await?;

    Ok(profile)
}

pub async fn get_all_profiles(db: &DatabaseConnection) -> Result<Vec<profile::Model>, AppError> {
    let profiles = profile::Entity::find()
        .order_by_desc(profile::Column::UpdatedAt)
        .all(db)
        .await?;

    Ok(profiles)
}

pub async fn upsert_profile(
    db: &DatabaseConnection,
    name: String,
    email: String,
    avatar_url: String,
    cookies: String,
) -> Result<profile::Model, AppError> {
    let existing = profile::Entity::find()
        .filter(
            profile::Column::Name
                .eq(&name)
                .or(profile::Column::GoogleId.eq(&name))
                .or(profile::Column::GoogleId.eq("No channel")),
        )
        .one(db)
        .await?;

    let now = Utc::now();

    if let Some(existing) = existing {
        let mut active: profile::ActiveModel = existing.into();
        active.google_id = Set(name.clone());
        active.name = Set(name);
        active.email = Set(email);
        active.avatar_url = Set(avatar_url);
        active.cookies = Set(cookies);
        active.updated_at = Set(now);

        let updated = active.update(db).await?;
        Ok(updated)
    } else {
        let count = profile::Entity::find().count(db).await?;
        let is_first = count == 0;

        let new_profile = profile::ActiveModel {
            google_id: Set(name.clone()),
            name: Set(name),
            email: Set(email),
            avatar_url: Set(avatar_url),
            cookies: Set(cookies),
            is_active: Set(is_first),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted = new_profile.insert(db).await?;
        Ok(inserted)
    }
}

pub async fn set_active_profile(
    db: &DatabaseConnection,
    id: i32,
) -> Result<profile::Model, AppError> {
    let txn = db.begin().await?;

    profile::Entity::update_many()
        .col_expr(
            profile::Column::IsActive,
            sea_orm::sea_query::Expr::value(false),
        )
        .exec(&txn)
        .await?;

    let target = profile::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Profile with id {id} not found")))?;

    let mut active: profile::ActiveModel = target.into();
    active.is_active = Set(true);
    active.updated_at = Set(Utc::now());

    let updated = active.update(&txn).await?;
    txn.commit().await?;

    Ok(updated)
}

pub async fn logout(db: &DatabaseConnection) -> Result<(), AppError> {
    profile::Entity::update_many()
        .col_expr(profile::Column::IsActive, false.into())
        .filter(profile::Column::IsActive.eq(true))
        .exec(db)
        .await
        .map_err(|e| AppError::internal(e))?;

    Ok(())
}

pub async fn delete_profile(db: &DatabaseConnection, id: i32) -> Result<bool, AppError> {
    let result = profile::Entity::delete_by_id(id).exec(db).await?;
    Ok(result.rows_affected > 0)
}
