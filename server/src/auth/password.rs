#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {}

impl Password {
    pub fn generate(credentials: String) -> Self {
        todo!()
    }
}
