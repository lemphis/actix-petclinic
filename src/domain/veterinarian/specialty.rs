use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "specialties")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: Option<String>,
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
