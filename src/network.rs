use std::{collections::HashMap, error::Error};

use uuid::Uuid;

use crate::{
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
        Ok(Self {
            name: String::from(name),
            id: certs.id,
            connection: C::new(certs, server),
            store,
            devices: HashMap::new(),
        })
    }

    pub fn create_device(&mut self, name: &str) -> &mut Device<'a> {
        self.devices.entry(String::from(name)).or_default()
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.start().await?;
        self.publish();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection.stop();
        self.store.save_schema(self.schema())?;
        Ok(())
    }

    fn publish(&mut self) {
        self.connection.send(
            Rpc::builder()
                .method(RpcMethod::POST)
                .on_type(RpcType::NETWORK)
                .data(self.schema())
                .create(),
        );
    }

    fn schema(&self) -> Schema {
        let mut schema = Schema::new(&self.name, self.id);
        schema.device = self
            .devices
            .iter()
            .map(|(name, device)| {
                let mut device_schema = DeviceSchema::new(name, device.id);
                device_schema.value = device
                    .values
                    .iter()
                    .map(|(name, value)| {
                        ValueSchema::new(
                            name.to_string(),
                            match (value.report.as_ref(), value.control.as_ref()) {
                                (Some(_), Some(_)) => Permission::RW,
                                (Some(_), None) => Permission::R,
                                (None, Some(_)) => Permission::W,
                                _ => panic!("Invalid permission"),
                            },
                            NumberSchema::new(0f64, 1f64, 1f64, "thingy"),
                        )
                    })
                    .collect();
                device_schema
            })
            .collect();
        schema
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
}

pub struct Device<'a> {
    pub id: Uuid,
    values: HashMap<String, Value<'a>>,
}

impl<'a> Device<'a> {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: HashMap::new(),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission<'a>) -> &mut Value<'a> {
        self.values
            .entry(String::from(name))
            .or_insert_with(|| Value::new(permission))
    }

    #[cfg(test)]
    pub fn values(&self) -> &HashMap<String, Value<'a>> {
        &self.values
    }
}

impl Default for Device<'_> {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}

#[allow(dead_code)]
pub struct Value<'a> {
    control: Option<ControlState<'a>>,
    report: Option<ReportState>,
}

impl<'a> Value<'a> {
    pub fn new(permission: ValuePermission<'a>) -> Self {
        let (report, control) = match permission {
            ValuePermission::RW(f) => (
                Some(ReportState::new(Uuid::new_v4())),
                Some(ControlState::new(Uuid::new_v4(), f)),
            ),
            ValuePermission::R => (Some(ReportState::new(Uuid::new_v4())), None),
            ValuePermission::W(f) => (None, Some(ControlState::new(Uuid::new_v4(), f))),
        };

        Self { report, control }
    }
    #[cfg(test)]
    pub fn control(&mut self, data: String) {
        (self.control.as_mut().unwrap().callback)(data)
    }
}

pub enum ValuePermission<'a> {
    RW(Box<dyn FnMut(String) + 'a>),
    R,
    W(Box<dyn FnMut(String) + 'a>),
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
