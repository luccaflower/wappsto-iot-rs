#[cfg(test)]
mod schema_builder_test {
    use crate::schema::*;
    use uuid::Uuid;

    #[test]
    fn can_build_an_empty_network_schema() {
        let id = Uuid::new_v4();
        let schema = SchemaBuilder::new(id).create();
        assert!(schema.device.is_empty())
    }
}
