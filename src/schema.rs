use serde::{Deserialize, Serialize};
use uuid::Uuid;

///A Schema represents the internal data structure of an IoT client as understood by Wappsto. These
///schemas are referred to as "networks", and they may contain devices, values for devices, as well
///as various kinds of metadata required by Wappsto. Network schemas can be generated
///programmatically using [SchemaBuilder] and [DeviceBuilder].
#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub meta: Meta,
    pub device: Vec<Device>,
}

impl Schema {
    pub fn new(id: Uuid) -> Self {
        Schema {
            name: String::new(),
            meta: Meta {
                id,
                meta_type: "network".to_owned(),
                version: "2.0".to_owned(),
            },
            device: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub value: Vec<Value>,
    pub meta: Meta,
}

impl Device {
    pub fn new() -> Self {
        Device {
            name: "".to_owned(),
            value: vec![],
            meta: Meta {
                id: Uuid::new_v4(),
                meta_type: String::from("device"),
                version: String::from("2.0"),
            },
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Value {
    pub name: String,
    pub permission: Permission,
    pub number: NumberSchema,
    pub state: Vec<State>,
    pub meta: Meta,
}

impl Value {
    pub fn new(name: String, permission: Permission, number: NumberSchema) -> Self {
        let state = match permission {
            Permission::R => vec![State::new(StateType::REPORT)],
            Permission::W => vec![State::new(StateType::CONTROL)],
            Permission::RW => vec![
                State::new(StateType::REPORT),
                State::new(StateType::CONTROL),
            ],
        };
        Value {
            name,
            permission,
            number,
            state,
            meta: Meta {
                id: Uuid::new_v4(),
                meta_type: String::from("value"),
                version: String::from("2.0"),
            },
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::new(
            String::from("State"),
            Permission::R,
            NumberSchema::default(),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    data: String,
    #[serde(rename = "type")]
    state_type: StateType,
    timestamp: String,
    meta: Meta,
}

impl State {
    pub fn new(state_type: StateType) -> Self {
        State {
            data: String::new(),
            state_type,
            timestamp: String::new(),
            meta: Meta {
                id: Uuid::new_v4(),
                meta_type: String::from("state"),
                version: String::from("2.0"),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum StateType {
    REPORT,
    CONTROL,
}

#[derive(Serialize, Deserialize)]
pub struct NumberSchema {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub unit: String,
}

impl NumberSchema {
    pub fn new(min: f64, max: f64, step: f64, unit: String) -> Self {
        NumberSchema {
            min,
            max,
            step,
            unit,
        }
    }
}

impl Default for NumberSchema {
    fn default() -> Self {
        NumberSchema::new(0f64, 1f64, 1f64, String::from(""))
    }
}

#[derive(Serialize, Deserialize)]
pub enum Permission {
    R,
    W,
    RW,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub meta_type: String,
    pub version: String,
}

pub struct SchemaBuilder {
    name: String,
    id: Uuid,
    device: Vec<Device>,
}

///Used to generate network schematics programmatically.
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use wappsto_iot_rs::schema::*;
///
/// //Network UUID should be provided by Wappsto
/// let example_uuid = Uuid::new_v4();
///
/// let schema = SchemaBuilder::new(example_uuid)
///    .named(String::from("test"))
///    .add_device(
///        DeviceBuilder::new()
///            .named(String::from("button"))
///            .add_value(Value::default())
///            .create(),
///    )
///    .create();
/// ```

impl SchemaBuilder {
    pub fn new(id: Uuid) -> Self {
        SchemaBuilder {
            name: "".to_owned(),
            id,
            device: vec![],
        }
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_device(mut self, device: Device) -> Self {
        self.device.push(device);
        self
    }

    pub fn create(self) -> Schema {
        Schema {
            name: self.name,
            meta: Meta {
                id: self.id,
                meta_type: String::from("network"),
                version: String::from("2.0"),
            },
            device: self.device,
        }
    }
}

pub struct DeviceBuilder {
    name: String,
    value: Vec<Value>,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        DeviceBuilder {
            name: String::new(),
            value: vec![],
        }
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_value(mut self, value: Value) -> Self {
        self.value.push(value);
        self
    }

    pub fn create(self) -> Device {
        Device {
            name: self.name,
            value: self.value,
            meta: Meta {
                id: Uuid::new_v4(),
                meta_type: String::from("device"),
                version: String::from("2.0"),
            },
        }
    }
}

impl Default for DeviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
