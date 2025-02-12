use crate::storage::ContactStore;

#[derive(Debug)]
pub struct InMemoryContactStore {}

impl ContactStore for InMemoryContactStore {}
