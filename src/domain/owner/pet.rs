use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "pets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: Option<String>,
    pub birth_date: Option<Date>,
    pub type_id: u32,
    pub owner_id: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::owners::Entity",
        from = "Column::OwnerId",
        to = "super::owners::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Owners,
    #[sea_orm(
        belongs_to = "super::types::Entity",
        from = "Column::TypeId",
        to = "super::types::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Types,
    #[sea_orm(has_many = "super::visit::Entity")]
    Visits,
}

impl Related<super::owners::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owners.def()
    }
}

impl Related<super::types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Types.def()
    }
}

impl Related<super::visit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Visits.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
