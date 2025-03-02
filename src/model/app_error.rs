use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Owner not found with id: {0}. Please ensure the ID is correct and the owner exists in the database.")]
    OwnerNotFound(u32),
}
