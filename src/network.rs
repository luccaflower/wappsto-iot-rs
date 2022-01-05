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
        println!("Cert ID: {}", certs.id);
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
        self.connection.stop();
        self.store.save_schema(self.schema())?;
        Ok(())
    }

    async fn publish(&mut self) {
        self.connection
            .send(
                Rpc::builder()
                    .method(RpcMethod::POST)
                    .on_type(RpcType::NETWORK)
                    .data(self.schema())
                    .create(),
            )
            .await;
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

    fn parse_schema(store: &S, certs: &Certs) -> HashMap<String, Device<'a>> {
        if let Some(schema) = store.load_schema(certs.id) {
            println!("Loading schema");
            schema
                .device
                .into_iter()
                .map(|d| {
                    println!(
                        "Adding device to schema. Id: {}, Name: {}",
                        d.meta.id, d.name
                    );
                    let mut device = Device::new(d.meta.id);
                    device.values = d
                        .value
                        .into_iter()
                        .map(|v| {
                            let value = Value::new(match v.permission {
                                Permission::R => ValuePermission::R,
                                Permission::W => ValuePermission::W(Box::new(|_| {})),
                                Permission::RW => ValuePermission::RW(Box::new(|_| {})),
                            });
                            (v.name, value)
                        })
                        .collect::<HashMap<String, Value>>();
                    (d.name, device)
                })
                .collect::<HashMap<String, Device>>()
        } else {
            println!("Schema not found");
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
