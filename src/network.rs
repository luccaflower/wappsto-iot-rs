use std::{collections::HashMap, error::Error};

use uuid::Uuid;

use crate::{
    certs::Certs,
    connection::{Connect, Connection, WappstoServers},
    fs_store::{FsStore, Store},
    rpc::{Rpc, RpcMethod, RpcType},
    schema::{DeviceSchema, NumberSchema, Permission, Schema, ValueSchema},
};

pub struct Network<'a, C = Connection, S = FsStore>
where
    C: Connect,
    S: Store + Default,
{
    pub name: String,
    pub id: Uuid,
    connection: C,
    store: S,
    devices: HashMap<String, Device<'a>>,
}

impl<'a, C, S> Network<'a, C, S>
where
    C: Connect,
    S: Store + Default,
{
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::new_at(WappstoServers::default(), name)
    }

    pub fn new_at(server: WappstoServers, name: &str) -> Result<Self, Box<dyn Error>> {
        let store = S::default();
        let certs = store.load_certs()?;
        let devices = Self::parse_schema(&store, &certs);
        Ok(Self {
            name: String::from(name),
            id: certs.id,
            connection: C::new(certs, server),
            store,
            devices,
        })
    }

    pub fn create_device(&mut self, name: &str) -> &mut Device<'a> {
        self.devices.entry(String::from(name)).or_default()
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.start().await?;
        self.publish().await;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let schema: Schema = self.into();
        self.connection.stop();
        self.store.save_schema(schema)?;
        Ok(())
    }

    async fn publish(&mut self) {
        let schema: Schema = self.into();
        self.connection
            .send(
                Rpc::builder()
                    .method(RpcMethod::Post)
                    .on_type(RpcType::Network)
                    .data(schema)
                    .create(),
            )
            .await;
    }

    fn parse_schema(store: &S, certs: &Certs) -> HashMap<String, Device<'a>> {
        if let Some(schema) = store.load_schema(certs.id) {
            schema
                .device
                .into_iter()
                .map(|d| (d.name.clone(), Device::from(d)))
                .collect::<HashMap<String, Device>>()
        } else {
            HashMap::new()
        }
    }

    #[cfg(test)]
    pub fn connection(&self) -> &C {
        &self.connection
    }

    #[cfg(test)]
    pub fn store(&self) -> &S {
        &self.store
    }

    #[cfg(test)]
    pub fn devices(&mut self) -> &mut HashMap<String, Device<'a>> {
        &mut self.devices
    }

    #[cfg(test)]
    pub fn new_with_store(name: &str, store: S) -> Self {
        let certs = store.load_certs().unwrap();
        let id = certs.id;
        let devices = Self::parse_schema(&store, &certs);
        Self {
            name: String::from(name),
            id,
            store,
            devices,
            connection: C::new(certs, WappstoServers::default()),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<C, S> Into<Schema> for &mut Network<'_, C, S>
where
    C: Connect,
    S: Store + Default,
{
    fn into(self) -> Schema {
        let mut schema = Schema::new(&self.name, self.id);
        schema.device = self
            .devices
            .iter()
            .map(|(_, device)| device.into())
            .collect();
        schema
    }
}

pub struct Device<'a> {
    pub name: String,
    pub id: Uuid,
    values: HashMap<String, Value<'a>>,
}

impl<'a> Device<'a> {
    pub fn new(name: &str, id: Uuid) -> Self {
        Self {
            name: String::from(name),
            id,
            values: HashMap::new(),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission<'a>) -> &mut Value<'a> {
        self.values
            .entry(String::from(name))
            .or_insert_with(|| Value::new(name, permission))
    }

    #[cfg(test)]
    pub fn values(&self) -> &HashMap<String, Value<'a>> {
        &self.values
    }
}

impl Default for Device<'_> {
    fn default() -> Self {
        Self::new("", Uuid::new_v4())
    }
}

#[allow(clippy::from_over_into)]
impl Into<DeviceSchema> for &Device<'_> {
    fn into(self) -> DeviceSchema {
        let mut schema = DeviceSchema::new(&self.name, self.id);
        schema.value = self.values.iter().map(|(_, value)| value.into()).collect();
        schema
    }
}

impl From<DeviceSchema> for Device<'_> {
    fn from(schema: DeviceSchema) -> Self {
        let mut device = Device::new(&schema.name, schema.meta.id);
        device.values = schema
            .value
            .into_iter()
            .map(|v| (v.name.clone(), Value::from(v)))
            .collect::<HashMap<String, Value>>();
        device
    }
}

pub struct Value<'a> {
    name: String,
    control: Option<ControlState<'a>>,
    report: Option<ReportState>,
}

impl<'a> Value<'a> {
    pub fn new(name: &str, permission: ValuePermission<'a>) -> Self {
        let (report, control) = match permission {
            ValuePermission::RW(f) => (
                Some(ReportState::new(Uuid::new_v4())),
                Some(ControlState::new(Uuid::new_v4(), f)),
            ),
            ValuePermission::R => (Some(ReportState::new(Uuid::new_v4())), None),
            ValuePermission::W(f) => (None, Some(ControlState::new(Uuid::new_v4(), f))),
        };

        Self {
            name: String::from(name),
            report,
            control,
        }
    }
    #[cfg(test)]
    pub fn control(&mut self, data: String) {
        (self.control.as_mut().unwrap().callback)(data)
    }
}

impl From<ValueSchema> for Value<'_> {
    fn from(schema: ValueSchema) -> Self {
        Self::new(&schema.name, ValuePermission::from(schema.permission))
    }
}

#[allow(clippy::from_over_into)]
impl Into<ValueSchema> for &Value<'_> {
    fn into(self) -> ValueSchema {
        let permission = match (self.report.as_ref(), self.control.as_ref()) {
            (Some(_), Some(_)) => Permission::RW,
            (Some(_), None) => Permission::R,
            (None, Some(_)) => Permission::W,
            _ => panic!("Invalid permission"),
        };
        ValueSchema::new(&self.name, permission, NumberSchema::default())
    }
}

pub enum ValuePermission<'a> {
    RW(Box<dyn FnMut(String) + 'a>),
    R,
    W(Box<dyn FnMut(String) + 'a>),
}

impl From<Permission> for ValuePermission<'_> {
    fn from(permission: Permission) -> Self {
        match permission {
            Permission::R => ValuePermission::R,
            Permission::RW => ValuePermission::RW(Box::new(|_| {})),
            Permission::W => ValuePermission::W(Box::new(|_| {})),
        }
    }
}

#[allow(dead_code)]
struct ControlState<'a> {
    pub id: Uuid,
    pub callback: Box<dyn FnMut(String) + 'a>,
}

#[allow(dead_code)]
struct ReportState {
    pub id: Uuid,
}

impl<'a> ControlState<'a> {
    pub fn new(id: Uuid, callback: Box<dyn FnMut(String) + 'a>) -> Self {
        Self { id, callback }
    }
}

impl ReportState {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}
