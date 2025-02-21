use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "visits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub pet_id: Option<u32>,
    pub visit_date: Option<Date>,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pet::Entity",
        from = "Column::PetId",
        to = "super::pet::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Pets,
}

impl Related<super::pet::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
