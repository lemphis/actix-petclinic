use std::collections::BTreeMap;

use sea_orm::{
    prelude::{Date, Expr},
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, FromQueryResult, JoinType,
    PaginatorTrait, QueryFilter, QuerySelect, RelationTrait,
};
use serde::Serialize;

use crate::{
    domain::owner::{owners, pet, types},
    model::app_error::AppError,
};

pub struct OwnerService;

#[derive(Serialize, FromQueryResult)]
struct OwnerWithPetsAndTypesQueryResult {
    owner_id: u32,
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
}

#[derive(Serialize)]
pub struct OwnerWithPetsAndTypes {
    owner_id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    telephone: Option<String>,
    pub pets_with_type: Vec<PetWithType>,
}

#[derive(Serialize)]
pub struct PetWithType {
    pub pet_id: u32,
    pub pet_name: Option<String>,
    birth_date: Option<Date>,
    type_id: u32,
    type_name: Option<String>,
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

    pub async fn fetch_owner_with_pets_and_types_by_owner_id(
        conn: &DbConn,
        owner_id: u32,
    ) -> Result<OwnerWithPetsAndTypes, AppError> {
        let owner_with_pets_and_types = owners::Entity::find_by_id(owner_id)
            .join(JoinType::LeftJoin, owners::Relation::Pets.def())
            .join(JoinType::LeftJoin, pet::Relation::Types.def())
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
            .into_model::<OwnerWithPetsAndTypesQueryResult>()
            .all(conn)
            .await?;

        if owner_with_pets_and_types.is_empty() {
            return Err(AppError::ResourceNotFound {
                resource: "owner".to_string(),
                id: owner_id,
            });
        }

        Ok(Self::group_owners_by_id(owner_with_pets_and_types).swap_remove(0))
    }

    fn group_owners_by_id(
        owners: Vec<OwnerWithPetsAndTypesQueryResult>,
    ) -> Vec<OwnerWithPetsAndTypes> {
        let mut owner_map: BTreeMap<u32, OwnerWithPetsAndTypes> = BTreeMap::new();
        for OwnerWithPetsAndTypesQueryResult {
            owner_id,
            first_name,
            last_name,
            address,
            city,
            telephone,
            pet_id,
            pet_name,
            birth_date,
            type_id,
            type_name,
        } in owners
        {
            let owner_entry = owner_map
                .entry(owner_id)
                .or_insert_with(|| OwnerWithPetsAndTypes {
                    owner_id,
                    first_name,
                    last_name,
                    address,
                    city,
                    telephone,
                    pets_with_type: Vec::new(),
                });

            if let Some(pet_id) = pet_id {
                owner_entry.pets_with_type.push(PetWithType {
                    pet_id,
                    pet_name,
                    birth_date,
                    type_id: type_id.unwrap(),
                    type_name,
                });
            }
        }

        owner_map.into_values().collect()
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
