use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
}
impl Schema {
    pub fn new(id: Uuid) -> Schema {
        Schema { id }
    }
}
