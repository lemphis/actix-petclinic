use dotenvy::dotenv;

pub fn load() {
    dotenv().ok();
}
