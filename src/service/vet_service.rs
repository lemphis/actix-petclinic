use std::collections::BTreeMap;

use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, FromQueryResult, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};
use serde::Serialize;

use crate::{
    domain::veterinarian::{specialty, vet, vet_specialty},
    model::app_error::AppError,
};

pub struct VetService;

#[derive(Serialize, FromQueryResult)]
struct VetWithSpecialtiesQueryResult {
    vet_id: u32,
    first_name: Option<String>,
    last_name: Option<String>,
    specialty_id: Option<u32>,
    specialty_name: Option<String>,
}

#[derive(Serialize)]
pub struct VetWithSpecialties {
    pub vet_id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub specialties: Vec<Specialty>,
}

#[derive(Serialize)]
pub struct Specialty {
    pub specialty_id: u32,
    pub specialty_name: Option<String>,
}

impl VetService {
    pub async fn fetch_all_vets_with_specialties(
        conn: &DbConn,
    ) -> Result<Vec<VetWithSpecialties>, AppError> {
        let all_vets_with_specialties = vet::Entity::find()
            .join(JoinType::LeftJoin, vet::Relation::VetSpecialties.def())
            .join(
                JoinType::LeftJoin,
                vet_specialty::Relation::Specialties.def(),
            )
            .select_only()
            .column_as(vet::Column::Id, "vet_id")
            .column(vet::Column::FirstName)
            .column(vet::Column::LastName)
            .column_as(specialty::Column::Id, "specialty_id")
            .column_as(specialty::Column::Name, "specialty_name")
            .order_by_asc(vet::Column::Id)
            .order_by_asc(specialty::Column::Id)
            .into_model::<VetWithSpecialtiesQueryResult>()
            .all(conn)
            .await?;

        Ok(Self::group_vets_by_id(all_vets_with_specialties))
    }

    fn group_vets_by_id(
        vets_with_specialties: Vec<VetWithSpecialtiesQueryResult>,
    ) -> Vec<VetWithSpecialties> {
        let mut vet_map: BTreeMap<u32, VetWithSpecialties> = BTreeMap::new();
        for VetWithSpecialtiesQueryResult {
            vet_id,
            first_name,
            last_name,
            specialty_id,
            specialty_name,
        } in vets_with_specialties
        {
            let vet_entry = vet_map.entry(vet_id).or_insert_with(|| VetWithSpecialties {
                vet_id,
                first_name,
                last_name,
                specialties: Vec::new(),
            });

            if let Some(specialty_id) = specialty_id {
                vet_entry.specialties.push(Specialty {
                    specialty_id,
                    specialty_name,
                });
            }
        }

        vet_map.into_values().collect()
    }

    pub async fn fetch_vets_with_specialties_paginated(
        conn: &DbConn,
        page: u64,
        size: u64,
    ) -> Result<Vec<VetWithSpecialties>, AppError> {
        let vet_ids: Vec<u32> = vet::Entity::find()
            .order_by_asc(vet::Column::Id)
            .paginate(conn, size)
            .fetch_page(page - 1)
            .await?
            .into_iter()
            .map(|vet| vet.id)
            .collect();

        let vets_with_specialties_paginated = vet::Entity::find()
            .join(JoinType::LeftJoin, vet::Relation::VetSpecialties.def())
            .join(
                JoinType::LeftJoin,
                vet_specialty::Relation::Specialties.def(),
            )
            .filter(vet::Column::Id.is_in(vet_ids))
            .select_only()
            .column_as(vet::Column::Id, "vet_id")
            .column(vet::Column::FirstName)
            .column(vet::Column::LastName)
            .column_as(specialty::Column::Id, "specialty_id")
            .column_as(specialty::Column::Name, "specialty_name")
            .into_model::<VetWithSpecialtiesQueryResult>()
            .all(conn)
            .await?;

        Ok(Self::group_vets_by_id(vets_with_specialties_paginated))
    }

    pub async fn fetch_all_vets_count(conn: &DbConn) -> Result<u64, AppError> {
        let vet_total_count = vet::Entity::find().count(conn).await?;

        Ok(vet_total_count)
    }
}
