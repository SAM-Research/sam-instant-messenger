#[derive(Clone)]
pub struct ServerState {}

impl ServerState {
    pub async fn init(&mut self) {}
    pub async fn cleanup(&mut self) {}
}
