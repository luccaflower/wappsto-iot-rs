use chrono::Utc;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    error::Error,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};
use uuid::Uuid;

use crate::{
    certs::Certs,
    communication::CallbackMap,
    connection::{Connect, Connection, SendChannel, WappstoServers, WrappedSend},
    fs_store::{FsStore, Store},
    rpc::{RpcData, RpcMethod, RpcRequest, RpcStateData, RpcType},
    schema::{
        DeviceSchema, Meta, MetaType, NumberSchema, Permission, Schema, State, StateType,
        ValueSchema,
    },
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

    pub fn create_device(&self, name: &str) -> Device<Se> {
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
    pub fn device_named(&self, name: &str) -> Option<Device<Se>> {
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
    devices: HashMap<String, Device<Se>>,
    pub send: Rc<RefCell<Option<Se>>>,
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
            send: Rc::new(RefCell::new(None)),
        })
    }

    pub fn create_device(&mut self, name: &str) -> Device<Se> {
        let device = self
            .devices
            .entry(String::from(name))
            .and_modify(|d| d.inner.borrow_mut().send = Rc::clone(&self.send))
            .or_insert_with(|| {
                Device::new(InnerDevice::new(
                    name,
                    Uuid::new_v4(),
                    Rc::clone(&self.send),
                ))
            });
        Device::clone(device)
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.send
            .borrow_mut()
            .replace(self.connection.start(self.callbacks())?);
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
        self.send
            .borrow()
            .as_ref()
            .unwrap()
            .send(serde_json::to_string(
                &RpcRequest::builder()
                    .method(RpcMethod::Post)
                    .on_type(RpcType::Network)
                    .data(RpcData::Schema(schema))
                    .create(),
            )?)?;
        Ok(())
    }

    fn parse_schema(store: &St, certs: &Certs) -> HashMap<String, Device<Se>> {
        if let Some(schema) = store.load_schema(certs.id) {
            schema
                .device
                .into_iter()
                .map(|d| (d.name.clone(), Device::from(d)))
                .collect::<HashMap<String, Device<Se>>>()
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
                        .inner
                        .lock()
                        .unwrap()
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
            send: Rc::new(RefCell::new(None)),
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

pub struct Device<Se: WrappedSend> {
    pub inner: Rc<RefCell<InnerDevice<Se>>>,
}

impl<Se: WrappedSend> Device<Se> {
    pub fn new(device: InnerDevice<Se>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(device)),
        }
    }

    pub fn create_value(&self, name: &str, permission: ValuePermission) -> Value<Se> {
        self.inner.borrow_mut().create_value(name, permission)
    }

    #[cfg(test)]
    pub fn value_named(&self, name: &str) -> Option<Value<Se>> {
        self.inner.borrow().value_named(name).cloned()
    }
}

impl<Se: WrappedSend> Clone for Device<Se> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<Se: WrappedSend> From<Ref<'_, InnerDevice<Se>>> for DeviceSchema {
    fn from(device: Ref<InnerDevice<Se>>) -> Self {
        let mut device_schema = DeviceSchema::new(&device.name, device.id);
        device_schema.value = device
            .values
            .iter()
            .map(|(_, value)| ValueSchema::from(value))
            .collect();
        device_schema
    }
}

impl<Se: WrappedSend> From<Device<Se>> for DeviceSchema {
    fn from(device: Device<Se>) -> Self {
        Self::from(device.inner.borrow())
    }
}

impl<Se: WrappedSend> From<DeviceSchema> for Device<Se> {
    fn from(schema: DeviceSchema) -> Self {
        Self::new(InnerDevice::from(schema))
    }
}

impl<Se: WrappedSend> Default for Device<Se> {
    fn default() -> Self {
        Self::new(InnerDevice::default())
    }
}

#[allow(dead_code)]
pub struct InnerDevice<Se: WrappedSend> {
    pub name: String,
    pub id: Uuid,
    values: HashMap<String, Value<Se>>,
    pub send: Rc<RefCell<Option<Se>>>,
}

impl<Se: WrappedSend> InnerDevice<Se> {
    pub fn new(name: &str, id: Uuid, send: Rc<RefCell<Option<Se>>>) -> Self {
        Self {
            name: String::from(name),
            id,
            values: HashMap::new(),
            send,
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn create_value(&mut self, name: &str, permission: ValuePermission) -> Value<Se> {
        let value = self
            .values
            .entry(String::from(name))
            .and_modify(|v| v.inner.lock().unwrap().send = Rc::clone(&self.send))
            .or_insert_with(|| {
                Value::new(InnerValue::new(name, permission, Rc::clone(&self.send)))
            });
        Value::clone(value)
    }

    #[cfg(test)]
    pub fn value_named(&self, key: &str) -> Option<&Value<Se>> {
        self.values.get(key)
    }
}

impl<Se: WrappedSend> Default for InnerDevice<Se> {
    fn default() -> Self {
        Self::new("", Uuid::new_v4(), Rc::new(RefCell::new(None)))
    }
}

impl<Se: WrappedSend> From<DeviceSchema> for InnerDevice<Se> {
    fn from(schema: DeviceSchema) -> Self {
        let mut device =
            InnerDevice::new(&schema.name, schema.meta.id, Rc::new(RefCell::new(None)));
        device.values = schema
            .value
            .into_iter()
            .map(|v| (v.name.clone(), Value::new(InnerValue::from(v))))
            .collect::<HashMap<String, Value<Se>>>();
        device
    }
}

impl<Se: WrappedSend> From<&InnerDevice<Se>> for DeviceSchema {
    fn from(device: &InnerDevice<Se>) -> Self {
        let mut device_schema = DeviceSchema::new(&device.name, device.id);
        device_schema.value = device
            .values
            .iter()
            .map(|(_, value)| ValueSchema::from(value))
            .collect();
        device_schema
    }
}

pub struct Value<Se: WrappedSend> {
    pub inner: Arc<Mutex<InnerValue<Se>>>,
}

impl<Se: WrappedSend> Clone for Value<Se> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Se: WrappedSend> Value<Se> {
    pub fn new(value: InnerValue<Se>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    pub fn report(&self, data: &str) {
        self.inner.lock().unwrap().report(data)
    }

    pub fn on_control(&self, callback: Box<dyn Fn(String) + Send + Sync>) {
        self.inner.lock().unwrap().on_control(callback)
    }

    #[cfg(test)]
    pub fn control_id(&self) -> Uuid {
        self.inner
            .lock()
            .unwrap()
            .control
            .as_ref()
            .unwrap()
            .inner
            .id
            .clone()
    }
}

impl<Se: WrappedSend> From<ValueSchema> for Value<Se> {
    fn from(schema: ValueSchema) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerValue::from(schema))),
        }
    }
}

impl<Se: WrappedSend> From<&Value<Se>> for ValueSchema {
    fn from(value: &Value<Se>) -> Self {
        Self::from(value.inner.lock().unwrap())
    }
}

pub struct InnerValue<Se: WrappedSend> {
    name: String,
    id: Uuid,
    permission: ValuePermission,
    pub send: Rc<RefCell<Option<Se>>>,
    pub control: Option<ControlState>,
    pub report: Option<InnerReportState>,
}

impl<Se: WrappedSend> InnerValue<Se> {
    pub fn new(name: &str, permission: ValuePermission, send: Rc<RefCell<Option<Se>>>) -> Self {
        Self::new_with_id(name, permission, Uuid::new_v4(), send)
    }

    pub fn new_with_id(
        name: &str,
        permission: ValuePermission,
        id: Uuid,
        send: Rc<RefCell<Option<Se>>>,
    ) -> Self {
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
            send,
        }
    }

    pub fn report(&self, data: &str) {
        self.send
            .borrow()
            .as_ref()
            .unwrap()
            .send(
                serde_json::to_string(
                    &RpcRequest::builder()
                        .method(RpcMethod::Put)
                        .on_type(RpcType::State)
                        .data(RpcData::Data(RpcStateData::new(
                            data,
                            Utc::now(),
                            Meta::new_with_uuid(self.report.as_ref().unwrap().id, MetaType::State),
                        )))
                        .create(),
                )
                .unwrap(),
            )
            .unwrap();
    }

    pub fn on_control(&self, callback: Box<dyn Fn(String) + Send + Sync>) {
        *self
            .control
            .as_ref()
            .unwrap()
            .callback
            .as_ref()
            .lock()
            .unwrap() = callback
    }

    #[cfg(test)]
    pub fn control(&self, data: String) {
        (self.control.as_ref().unwrap().callback.lock().unwrap())(data)
    }
}

impl<Se: WrappedSend> From<ValueSchema> for InnerValue<Se> {
    fn from(schema: ValueSchema) -> Self {
        Self::new_with_id(
            &schema.name,
            ValuePermission::from(schema.permission),
            schema.meta.id,
            Rc::new(RefCell::new(None)),
        )
    }
}

impl<Se: WrappedSend> From<MutexGuard<'_, InnerValue<Se>>> for ValueSchema {
    fn from(value: MutexGuard<InnerValue<Se>>) -> Self {
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
    inner: Rc<InnerControlState>,
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

impl Deref for ControlState {
    type Target = InnerControlState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[allow(clippy::type_complexity)]
pub struct InnerControlState {
    pub id: Uuid,
    pub callback: Arc<Mutex<Box<dyn Fn(String) + Send + Sync>>>,
}

pub struct ReportState {
    inner: Rc<InnerReportState>,
}

impl ReportState {
    pub fn new(report_state: InnerReportState) -> Self {
        Self {
            inner: Rc::new(report_state),
        }
    }
}

impl Deref for ReportState {
    type Target = InnerReportState;
    fn deref(&self) -> &Self::Target {
        &self.inner
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
