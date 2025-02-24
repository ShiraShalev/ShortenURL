use rand::{distr::Alphanumeric, Rng};

pub fn generate_key() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}
