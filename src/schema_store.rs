use uuid::Uuid;

use crate::schema::Schema;

pub fn save(_schema: Schema) {}
pub fn load(_id: Uuid) -> Option<Schema> {
    None
}
