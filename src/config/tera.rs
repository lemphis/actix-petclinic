use tera::Tera;

pub fn init() -> Tera {
    Tera::new("templates/**/*.html").expect("Tera parsing error")
}
