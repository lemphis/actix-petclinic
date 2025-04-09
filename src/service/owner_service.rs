use std::collections::HashMap;

use sea_orm::{
    prelude::{Date, Expr},
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, FromQueryResult, JoinType,
    PaginatorTrait, QueryFilter, QuerySelect, RelationTrait,
};
use serde::Serialize;

use crate::{
    domain::owner::{owners, pet, types, visit},
    model::app_error::AppError,
};

pub struct OwnerService;

#[derive(Serialize, FromQueryResult)]
pub struct OwnerWithPetsAndTypesAndVisitsQueryResult {
    pub owner_id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    telephone: Option<String>,
    pet_id: Option<u32>,
    pet_name: Option<String>,
    birth_date: Option<Date>,
    type_id: Option<u32>,
    type_name: Option<String>,
    visit_id: Option<u32>,
    visit_date: Option<Date>,
    description: Option<String>,
}

#[derive(Serialize)]
pub struct OwnerWithPetsAndTypesAndVisits {
    owner_id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    telephone: Option<String>,
    pub pets_with_type: Vec<PetWithTypeAndVisits>,
}

#[derive(Serialize)]
pub struct PetWithTypeAndVisits {
    pub pet_id: u32,
    pub pet_name: Option<String>,
    pub birth_date: Option<Date>,
    pub pet_type: PetType,
    visits: Vec<Visit>,
}

#[derive(Serialize)]
pub struct PetType {
    type_id: u32,
    pub type_name: Option<String>,
}

#[derive(Serialize)]
pub struct Visit {
    visit_id: u32,
    visit_date: Option<Date>,
    description: Option<String>,
}

#[derive(Serialize, FromQueryResult)]
pub struct OwnersWithPetNames {
    pub id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    telephone: Option<String>,
    pet_names: Option<String>,
}

impl OwnerService {
    pub async fn fetch_owner_by_id(
        conn: &DbConn,
        owner_id: u32,
    ) -> Result<owners::Model, AppError> {
        owners::Entity::find_by_id(owner_id)
            .one(conn)
            .await?
            .ok_or_else(|| AppError::ResourceNotFound {
                resource: "owner".to_string(),
                id: owner_id,
            })
    }

    pub async fn fetch_owner_with_pets_and_types_and_visits_by_owner_id(
        conn: &DbConn,
        owner_id: u32,
    ) -> Result<OwnerWithPetsAndTypesAndVisits, AppError> {
        let rows = owners::Entity::find_by_id(owner_id)
            .join(JoinType::LeftJoin, owners::Relation::Pets.def())
            .join(JoinType::LeftJoin, pet::Relation::Types.def())
            .join(JoinType::LeftJoin, pet::Relation::Visits.def())
            .select_only()
            .column_as(owners::Column::Id, "owner_id")
            .column(owners::Column::FirstName)
            .column(owners::Column::LastName)
            .column(owners::Column::Address)
            .column(owners::Column::City)
            .column(owners::Column::Telephone)
            .column_as(pet::Column::Id, "pet_id")
            .column_as(pet::Column::Name, "pet_name")
            .column(pet::Column::BirthDate)
            .column_as(types::Column::Id, "type_id")
            .column_as(types::Column::Name, "type_name")
            .column_as(visit::Column::Id, "visit_id")
            .column_as(visit::Column::VisitDate, "visit_date")
            .column_as(visit::Column::Description, "description")
            .into_model::<OwnerWithPetsAndTypesAndVisitsQueryResult>()
            .all(conn)
            .await?;

        if rows.is_empty() {
            return Err(AppError::ResourceNotFound {
                resource: "owner".to_string(),
                id: owner_id,
            });
        }

        Ok(Self::transform_query_results(rows))
    }

    fn transform_query_results(
        rows: Vec<OwnerWithPetsAndTypesAndVisitsQueryResult>,
    ) -> OwnerWithPetsAndTypesAndVisits {
        let first_row = rows.first().unwrap(); // 함수 실행 전 rows가 empty인 경우를 filter 하기 때문에 반드시 Some임

        let pets_with_type = Self::group_pets_and_visits(&rows);

        OwnerWithPetsAndTypesAndVisits {
            owner_id: first_row.owner_id,
            first_name: first_row.first_name.clone(),
            last_name: first_row.last_name.clone(),
            address: first_row.address.clone(),
            city: first_row.city.clone(),
            telephone: first_row.telephone.clone(),
            pets_with_type,
        }
    }

    fn group_pets_and_visits(
        rows: &[OwnerWithPetsAndTypesAndVisitsQueryResult],
    ) -> Vec<PetWithTypeAndVisits> {
        let mut pets_and_visits: Vec<PetWithTypeAndVisits> = rows
            .iter()
            .filter_map(|row| row.pet_id.map(|pet_id| (pet_id, row)))
            .fold(
                HashMap::<u32, Vec<&OwnerWithPetsAndTypesAndVisitsQueryResult>>::new(),
                |mut acc, (pet_id, row)| {
                    acc.entry(pet_id).or_default().push(row);
                    acc
                },
            )
            .into_iter()
            .map(|(pet_id, pet_rows)| Self::create_pet_with_type_and_visits(pet_id, &pet_rows))
            .collect();

        pets_and_visits.sort_by_key(|p| p.pet_name.clone());

        pets_and_visits
    }

    fn create_pet_with_type_and_visits(
        pet_id: u32,
        pet_rows: &[&OwnerWithPetsAndTypesAndVisitsQueryResult],
    ) -> PetWithTypeAndVisits {
        let first_pet_row = pet_rows.first().unwrap(); // 함수 실행 전 pet_id가 존재할 때만 실행되므로 반드시 Some임
        let mut visits: Vec<Visit> = pet_rows
            .iter()
            .filter_map(|r| {
                r.visit_id.map(|visit_id| Visit {
                    visit_id,
                    visit_date: r.visit_date,
                    description: r.description.clone(),
                })
            })
            .collect();

        visits.sort_by_key(|v| v.visit_date);

        PetWithTypeAndVisits {
            pet_id,
            pet_name: first_pet_row.pet_name.clone(),
            birth_date: first_pet_row.birth_date,
            pet_type: PetType {
                type_id: first_pet_row.type_id.unwrap(),
                type_name: first_pet_row.type_name.clone(),
            },
            visits,
        }
    }

    pub async fn save_owner(
        conn: &DbConn,
        first_name: Option<String>,
        last_name: Option<String>,
        address: Option<String>,
        city: Option<String>,
        telephone: Option<String>,
    ) -> Result<owners::Model, AppError> {
        let owner_active_model = owners::ActiveModel {
            first_name: ActiveValue::Set(first_name),
            last_name: ActiveValue::Set(last_name),
            address: ActiveValue::Set(address),
            city: ActiveValue::Set(city),
            telephone: ActiveValue::Set(telephone),
            ..Default::default()
        };

        let new_owner = owner_active_model.insert(conn).await?;

        Ok(new_owner)
    }

    pub async fn fetch_owners_with_pet_names(
        conn: &DbConn,
        last_name: &str,
        page: u64,
        size: u64,
    ) -> Result<Vec<OwnersWithPetNames>, AppError> {
        let owners_with_pet_names = owners::Entity::find()
            .left_join(pet::Entity)
            .filter(owners::Column::LastName.like(format!("{}%", last_name)))
            .column_as(
                Expr::cust("GROUP_CONCAT(pets.name SEPARATOR ', ')"),
                "pet_names",
            )
            .group_by(owners::Column::Id)
            .into_model::<OwnersWithPetNames>()
            .paginate(conn, size)
            .fetch_page(page - 1)
            .await?;

        Ok(owners_with_pet_names)
    }

    pub async fn fetch_owner_count_by_last_name_prefix(
        conn: &DbConn,
        last_name: &str,
    ) -> Result<u64, AppError> {
        let owner_count = owners::Entity::find()
            .filter(owners::Column::LastName.like(format!("{last_name}%")))
            .count(conn)
            .await?;

        Ok(owner_count)
    }

    pub async fn update_owner(
        conn: &DbConn,
        owner_id: u32,
        first_name: Option<String>,
        last_name: Option<String>,
        address: Option<String>,
        city: Option<String>,
        telephone: Option<String>,
    ) -> Result<owners::Model, AppError> {
        let owner_active_model = owners::ActiveModel {
            id: ActiveValue::Unchanged(owner_id),
            first_name: ActiveValue::Set(first_name),
            last_name: ActiveValue::Set(last_name),
            address: ActiveValue::Set(address),
            city: ActiveValue::Set(city),
            telephone: ActiveValue::Set(telephone),
        };

        let updated_owner = owner_active_model.update(conn).await?;

        Ok(updated_owner)
    }
}
