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

    #[test]
    fn can_name_the_network() {
        let id = Uuid::new_v4();
        let schema = SchemaBuilder::new(id).named("test".to_owned()).create();
        assert_eq!("test", schema.name)
    }

    #[test]
    fn can_add_device_to_network() {
        let id = Uuid::new_v4();
        let schema = SchemaBuilder::new(id).add_device(Device::new()).create();
        assert!(!schema.device.is_empty())
    }
}
