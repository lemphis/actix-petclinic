use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "vets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::vet_specialty::Entity")]
    VetSpecialties,
}

impl Related<super::vet_specialty::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VetSpecialties.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
