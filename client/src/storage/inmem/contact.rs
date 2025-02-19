use crate::storage::ContactStore;

#[derive(Debug, Default)]
pub struct InMemoryContactStore {}

impl ContactStore for InMemoryContactStore {}
