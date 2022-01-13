use std::{cell::RefCell, collections::HashMap, error::Error};
use uuid::Uuid;

use crate::{
    certs::Certs,
    communication::CallbackMap,
    connection::{Connect, Connection, SendChannel, WappstoServers, WrappedSend},
    fs_store::{FsStore, Store},
    rpc::{RpcData, RpcMethod, RpcRequest, RpcType},
    schema::{DeviceSchema, NumberSchema, Permission, Schema, ValueSchema},
};

pub struct Network<C = Connection, St = FsStore, Se = SendChannel>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
{
    pub name: String,
    pub id: Uuid,
    connection: C,
    store: St,
    devices: HashMap<String, Device>,
    pub send: Option<Se>,
}

impl<C, St, Se> Network<C, St, Se>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
{
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::new_at(WappstoServers::default(), name)
    }

    pub fn new_at(server: WappstoServers, name: &str) -> Result<Self, Box<dyn Error>> {
        let store = St::default();
        let certs = store.load_certs()?;
        let devices = Self::parse_schema(&store, &certs);
        Ok(Self {
            name: String::from(name),
            id: certs.id,
            connection: C::new(certs, server),
            store,
            devices,
            send: None,
        })
    }

    pub fn create_device(&mut self, name: &str) -> &mut Device {
        self.devices.entry(String::from(name)).or_default()
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.send = Some(self.connection.start(self.callbacks())?);
        self.publish()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let schema: Schema = self.into();
        self.store.save_schema(schema)?;
        Ok(())
    }

    fn publish(&mut self) -> Result<(), Box<dyn Error>> {
        let schema: Schema = self.into();
        self.send.as_ref().unwrap().send(serde_json::to_string(
            &RpcRequest::builder()
                .method(RpcMethod::Post)
                .on_type(RpcType::Network)
                .data(RpcData::Schema(schema))
                .create(),
        )?)?;
        Ok(())
    }

    fn parse_schema(store: &St, certs: &Certs) -> HashMap<String, Device> {
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

    fn callbacks(&self) -> CallbackMap {
        self.devices
            .iter()
            .fold(HashMap::new(), |mut all_callbacks, (_, device)| {
                device.values.iter().for_each(|(_, value)| {
                    value
                        .control
                        .borrow_mut()
                        .take()
                        .and_then(|c| all_callbacks.insert(c.id, c.callback));
                });
                all_callbacks
            })
    }

    #[cfg(test)]
    pub fn connection(&mut self) -> &mut C {
        &mut self.connection
    }

    #[cfg(test)]
    pub fn store(&self) -> &St {
        &self.store
    }

    #[cfg(test)]
    pub fn devices(&mut self) -> &mut HashMap<String, Device> {
        &mut self.devices
    }

    #[cfg(test)]
    pub fn new_with_store(name: &str, store: St) -> Self {
        let certs = store.load_certs().unwrap();
        let id = certs.id;
        let devices = Self::parse_schema(&store, &certs);
        Self {
            name: String::from(name),
            id,
            store,
            devices,
            connection: C::new(certs, WappstoServers::default()),
            send: None,
        }
    }
}

#[allow(clippy::from_over_into)]
impl<C, St, Se> Into<Schema> for &mut Network<C, St, Se>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
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

pub struct Device {
    pub name: String,
    pub id: Uuid,
    values: HashMap<String, Value>,
}

impl Device {
    pub fn new(name: &str, id: Uuid) -> Self {
        Self {
            name: String::from(name),
            id,
            values: HashMap::new(),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission) -> &mut Value {
        self.values
            .entry(String::from(name))
            .or_insert_with(|| Value::new(name, permission))
    }

    #[cfg(test)]
    pub fn values(&self) -> &HashMap<String, Value> {
        &self.values
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new("", Uuid::new_v4())
    }
}

#[allow(clippy::from_over_into)]
impl Into<DeviceSchema> for &Device {
    fn into(self) -> DeviceSchema {
        let mut schema = DeviceSchema::new(&self.name, self.id);
        schema.value = self.values.iter().map(|(_, value)| value.into()).collect();
        schema
    }
}

impl From<DeviceSchema> for Device {
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

pub struct Value {
    name: String,
    id: Uuid,
    pub control: RefCell<Option<ControlState>>,
    report: Option<ReportState>,
}

impl Value {
    pub fn new(name: &str, permission: ValuePermission) -> Self {
        Self::new_with_id(name, permission, Uuid::new_v4())
    }

    pub fn new_with_id(name: &str, permission: ValuePermission, id: Uuid) -> Self {
        let (report, control) = match permission {
            ValuePermission::RW(f) => (
                Some(ReportState::new(Uuid::new_v4())),
                RefCell::new(Some(ControlState::new(Uuid::new_v4(), f))),
            ),
            ValuePermission::R => (Some(ReportState::new(Uuid::new_v4())), RefCell::new(None)),
            ValuePermission::W(f) => (
                None,
                RefCell::new(Some(ControlState::new(Uuid::new_v4(), f))),
            ),
        };

        Self {
            name: String::from(name),
            id,
            report,
            control,
        }
    }

    #[cfg(test)]
    pub fn control(&mut self, data: String) {
        (self.control.borrow_mut().as_mut().unwrap().callback)(data)
    }
}

impl From<ValueSchema> for Value {
    fn from(schema: ValueSchema) -> Self {
        Self::new_with_id(
            &schema.name,
            ValuePermission::from(schema.permission),
            schema.meta.id,
        )
    }
}

#[allow(clippy::from_over_into)]
impl Into<ValueSchema> for &Value {
    fn into(self) -> ValueSchema {
        let permission = match (self.report.as_ref(), self.control.borrow().as_ref()) {
            (Some(_), Some(_)) => Permission::RW,
            (Some(_), None) => Permission::R,
            (None, Some(_)) => Permission::W,
            _ => panic!("Invalid permission"),
        };
        ValueSchema::new_with_id(&self.name, permission, NumberSchema::default(), self.id)
    }
}

pub enum ValuePermission {
    RW(Box<dyn FnMut(String) + Send + Sync>),
    R,
    W(Box<dyn FnMut(String) + Send + Sync>),
}

impl From<Permission> for ValuePermission {
    fn from(permission: Permission) -> Self {
        match permission {
            Permission::R => ValuePermission::R,
            Permission::RW => ValuePermission::RW(Box::new(|_| {})),
            Permission::W => ValuePermission::W(Box::new(|_| {})),
        }
    }
}

#[allow(dead_code)]
pub struct ControlState {
    pub id: Uuid,
    pub callback: Box<dyn FnMut(String) + Send + Sync>,
}

#[allow(dead_code)]
struct ReportState {
    pub id: Uuid,
}

impl ControlState {
    pub fn new(id: Uuid, callback: Box<dyn FnMut(String) + Send + Sync>) -> Self {
        Self { id, callback }
    }
}

impl ReportState {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}
