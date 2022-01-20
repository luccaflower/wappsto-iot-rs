use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    error::Error,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::{
    certs::Certs,
    communication::CallbackMap,
    connection::{Connect, Connection, SendChannel, WappstoServers, WrappedSend},
    fs_store::{FsStore, Store},
    rpc::{RpcData, RpcMethod, RpcRequest, RpcType},
    schema::{DeviceSchema, NumberSchema, Permission, Schema, State, StateType, ValueSchema},
};

pub struct Network<C = Connection, St = FsStore, Se = SendChannel>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
{
    pub inner: Rc<RefCell<InnerNetwork<C, St, Se>>>,
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
        let inner = InnerNetwork::new_at(server, name)?;
        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
        })
    }

    pub fn create_device(&self, name: &str) -> Device {
        self.inner.borrow_mut().create_device(name)
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        self.inner.borrow_mut().start()
    }

    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        self.inner.borrow_mut().stop()
    }

    #[cfg(test)]
    pub fn new_with_store(name: &str, store: St) -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerNetwork::new_with_store(name, store))),
        }
    }

    #[cfg(test)]
    pub fn connection(&self) -> Rc<C> {
        self.inner.borrow().connection()
    }

    #[cfg(test)]
    pub fn store(&self) -> Rc<St> {
        self.inner.borrow().store()
    }

    #[cfg(test)]
    pub fn device_named(&self, name: &str) -> Option<Device> {
        self.inner.borrow().devices.get(name).cloned()
    }

    #[cfg(test)]
    pub fn devices_is_empty(&self) -> bool {
        self.inner.borrow().devices.is_empty()
    }

    #[cfg(test)]
    pub fn id(&self) -> Uuid {
        self.inner.borrow().id.clone()
    }
}

pub struct InnerNetwork<C = Connection, St = FsStore, Se = SendChannel>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
{
    pub name: String,
    pub id: Uuid,
    connection: Rc<C>,
    store: Rc<St>,
    devices: HashMap<String, Device>,
    pub send: Option<Se>,
}

impl<C, St, Se> InnerNetwork<C, St, Se>
where
    C: Connect<Se>,
    St: Store + Default,
    Se: WrappedSend,
{
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::new_at(WappstoServers::default(), name)
    }

    pub fn new_at(server: WappstoServers, name: &str) -> Result<Self, Box<dyn Error>> {
        let store = Rc::new(St::default());
        let certs = store.load_certs()?;
        let devices = Self::parse_schema(&store, &certs);
        Ok(Self {
            name: String::from(name),
            id: certs.id,
            connection: Rc::new(C::new(certs, server)),
            store,
            devices,
            send: None,
        })
    }

    pub fn create_device(&mut self, name: &str) -> Device {
        let device = self.devices.entry(String::from(name)).or_default();
        Device::clone(device)
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
                device.inner.borrow().values.iter().for_each(|(_, value)| {
                    value
                        .control
                        .as_ref()
                        .and_then(|c| all_callbacks.insert(c.inner.id, c.inner.callback.clone()));
                });
                all_callbacks
            })
    }

    #[cfg(test)]
    pub fn connection(&self) -> Rc<C> {
        Rc::clone(&self.connection)
    }

    #[cfg(test)]
    pub fn store(&self) -> Rc<St> {
        Rc::clone(&self.store)
    }

    #[cfg(test)]
    pub fn new_with_store(name: &str, store: St) -> Self {
        let certs = store.load_certs().unwrap();
        let id = certs.id;
        let devices = Self::parse_schema(&store, &certs);
        Self {
            name: String::from(name),
            id,
            store: Rc::new(store),
            devices,
            connection: Rc::new(C::new(certs, WappstoServers::default())),
            send: None,
        }
    }
}

#[allow(clippy::from_over_into)]
impl<C, St, Se> Into<Schema> for &mut InnerNetwork<C, St, Se>
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
            .map(|(_, device)| Device::clone(device).into())
            .collect();
        schema
    }
}

pub struct Device {
    pub inner: Rc<RefCell<InnerDevice>>,
}

impl Device {
    pub fn new(device: InnerDevice) -> Self {
        Self {
            inner: Rc::new(RefCell::new(device)),
        }
    }

    pub fn create_value(&self, name: &str, permission: ValuePermission) -> Value {
        self.inner.borrow_mut().create_value(name, permission)
    }

    #[cfg(test)]
    pub fn value_named(&self, name: &str) -> Option<Value> {
        self.inner.borrow().value_named(name).cloned()
    }
}

impl Clone for Device {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl From<Ref<'_, InnerDevice>> for DeviceSchema {
    fn from(device: Ref<InnerDevice>) -> Self {
        let mut device_schema = DeviceSchema::new(&device.name, device.id);
        device_schema.value = device
            .values
            .iter()
            .map(|(_, value)| ValueSchema::from(value))
            .collect();
        device_schema
    }
}

impl From<Device> for DeviceSchema {
    fn from(device: Device) -> Self {
        Self::from(device.inner.borrow())
    }
}

impl From<DeviceSchema> for Device {
    fn from(schema: DeviceSchema) -> Self {
        Self::new(InnerDevice::from(schema))
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new(InnerDevice::default())
    }
}

pub struct InnerDevice {
    pub name: String,
    pub id: Uuid,
    values: HashMap<String, Value>,
}

impl InnerDevice {
    pub fn new(name: &str, id: Uuid) -> Self {
        Self {
            name: String::from(name),
            id,
            values: HashMap::new(),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission) -> Value {
        let value = Value::new(InnerValue::new(name, permission));
        self.values
            .entry(String::from(name))
            .or_insert_with(|| Value::clone(&value));
        value
    }

    #[cfg(test)]
    pub fn value_named(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl Default for InnerDevice {
    fn default() -> Self {
        Self::new("", Uuid::new_v4())
    }
}

impl From<DeviceSchema> for InnerDevice {
    fn from(schema: DeviceSchema) -> Self {
        let mut device = InnerDevice::new(&schema.name, schema.meta.id);
        device.values = schema
            .value
            .into_iter()
            .map(|v| (v.name.clone(), Value::new(InnerValue::from(v))))
            .collect::<HashMap<String, Value>>();
        device
    }
}

impl From<&InnerDevice> for DeviceSchema {
    fn from(device: &InnerDevice) -> Self {
        let mut device_schema = DeviceSchema::new(&device.name, device.id);
        device_schema.value = device
            .values
            .iter()
            .map(|(_, value)| ValueSchema::from(value))
            .collect();
        device_schema
    }
}

pub struct Value {
    pub inner: Rc<InnerValue>,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl Value {
    pub fn new(value: InnerValue) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }

    #[cfg(test)]
    pub fn control_id(&self) -> Uuid {
        self.control.as_ref().unwrap().inner.id.clone()
    }
}

impl Deref for Value {
    type Target = InnerValue;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl From<ValueSchema> for Value {
    fn from(schema: ValueSchema) -> Self {
        Self {
            inner: Rc::new(InnerValue::from(schema)),
        }
    }
}

impl From<&Value> for ValueSchema {
    fn from(value: &Value) -> Self {
        Self::from(value.inner.clone().as_ref())
    }
}

pub struct InnerValue {
    name: String,
    id: Uuid,
    permission: ValuePermission,
    pub control: Option<ControlState>,
    pub report: Option<InnerReportState>,
}

impl InnerValue {
    pub fn new(name: &str, permission: ValuePermission) -> Self {
        Self::new_with_id(name, permission, Uuid::new_v4())
    }

    pub fn new_with_id(name: &str, permission: ValuePermission, id: Uuid) -> Self {
        let permission_record = match &permission {
            ValuePermission::R => ValuePermission::R,
            ValuePermission::RW(_) => ValuePermission::RW(Box::new(|_| {})),
            ValuePermission::W(_) => ValuePermission::W(Box::new(|_| {})),
        };
        let (report, control) = match permission {
            ValuePermission::RW(f) => (
                Some(InnerReportState::new(Uuid::new_v4())),
                Some(ControlState::new(InnerControlState::new(
                    Uuid::new_v4(),
                    Arc::new(Mutex::new(f)),
                ))),
            ),
            ValuePermission::R => (Some(InnerReportState::new(Uuid::new_v4())), None),
            ValuePermission::W(f) => (
                None,
                Some(ControlState::new(InnerControlState::new(
                    Uuid::new_v4(),
                    Arc::new(Mutex::new(f)),
                ))),
            ),
        };

        Self {
            name: String::from(name),
            id,
            permission: permission_record,
            report,
            control,
        }
    }

    pub fn report(&self, _data: &str) {
        todo!("report state")
    }

    #[cfg(test)]
    pub fn control(&self, data: String) {
        (self
            .control
            .as_ref()
            .unwrap()
            .inner
            .callback
            .lock()
            .unwrap())(data)
    }
}

impl From<ValueSchema> for InnerValue {
    fn from(schema: ValueSchema) -> Self {
        Self::new_with_id(
            &schema.name,
            ValuePermission::from(schema.permission),
            schema.meta.id,
        )
    }
}

impl From<&InnerValue> for ValueSchema {
    fn from(value: &InnerValue) -> Self {
        let permission = &value.permission;
        let permission: Permission = permission.into();
        let mut values_schema =
            Self::new_with_id(&value.name, permission, NumberSchema::default(), value.id);
        values_schema.state = vec![];
        if let Some(s) = value.report.as_ref() {
            values_schema
                .state
                .push(State::new_with_id(StateType::Report, s.id))
        };

        if let Some(s) = value.control.as_ref() {
            values_schema
                .state
                .push(State::new_with_id(StateType::Control, s.inner.id))
        };
        values_schema
    }
}

pub enum ValuePermission {
    RW(Box<dyn Fn(String) + Send + Sync>),
    R,
    W(Box<dyn Fn(String) + Send + Sync>),
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

#[allow(clippy::from_over_into)]
impl Into<Permission> for &ValuePermission {
    fn into(self) -> Permission {
        match self {
            ValuePermission::R => Permission::R,
            ValuePermission::RW(_) => Permission::RW,
            ValuePermission::W(_) => Permission::W,
        }
    }
}

pub struct ControlState {
    pub inner: Rc<InnerControlState>,
}

impl Clone for ControlState {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ControlState {
    pub fn new(control_state: InnerControlState) -> Self {
        Self {
            inner: Rc::new(control_state),
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct InnerControlState {
    pub id: Uuid,
    pub callback: Arc<Mutex<Box<dyn Fn(String) + Send + Sync>>>,
}

#[allow(dead_code)]
pub struct ReportState {
    pub inner: Rc<InnerReportState>,
}

impl ReportState {
    pub fn new(report_state: InnerReportState) -> Self {
        Self {
            inner: Rc::new(report_state),
        }
    }
}

pub struct InnerReportState {
    pub id: Uuid,
}

impl InnerControlState {
    #[allow(clippy::type_complexity)]
    pub fn new(id: Uuid, callback: Arc<Mutex<Box<dyn Fn(String) + Send + Sync>>>) -> Self {
        Self { id, callback }
    }
}

impl InnerReportState {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}
