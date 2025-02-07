use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "vet_specialties")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub vet_id: u32,
    #[sea_orm(primary_key)]
    pub specialty_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::specialty::Entity",
        from = "Column::SpecialtyId",
        to = "super::specialty::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Specialties,
    #[sea_orm(
        belongs_to = "super::vet::Entity",
        from = "Column::VetId",
        to = "super::vet::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Vets,
}

impl Related<super::specialty::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Specialties.def()
    }
}

impl Related<super::vet::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
