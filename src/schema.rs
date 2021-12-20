use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

///A Schema represents the internal data structure of an IoT client as understood by Wappsto. These
///schemas are referred to as "networks", and they may contain devices, values for devices, as well
///as various kinds of metadata required by Wappsto. Network schemas can be generated
///programmatically using [SchemaBuilder] and [DeviceBuilder]. The full JSON schematic can be found
///[here](https://wappsto.com/services/2.0/network/schema).
#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub meta: Meta,
    pub device: Vec<DeviceSchema>,
}

impl Schema {
    pub fn new(name: &str, id: Uuid) -> Self {
        Schema {
            name: name.to_owned(),
            meta: Meta::new_with_uuid(id, MetaType::NETWORK),
            device: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeviceSchema {
    pub name: String,
    pub value: Vec<ValueSchema>,
    pub meta: Meta,
}

impl DeviceSchema {
    pub fn new(name: &str, id: Uuid) -> Self {
        DeviceSchema {
            name: name.to_owned(),
            value: vec![],
            meta: Meta::new_with_uuid(id, MetaType::DEVICE),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValueSchema {
    pub name: String,
    pub permission: Permission,
    pub number: NumberSchema,
    pub state: Vec<State>,
    pub meta: Meta,
}

impl ValueSchema {
    pub fn new(name: String, permission: Permission, number: NumberSchema) -> Self {
        let state = match permission {
            Permission::R => vec![State::new(StateType::REPORT)],
            Permission::W => vec![State::new(StateType::CONTROL)],
            Permission::RW => vec![
                State::new(StateType::REPORT),
                State::new(StateType::CONTROL),
            ],
        };
        ValueSchema {
            name,
            permission,
            number,
            state,
            meta: Meta::new(MetaType::VALUE),
        }
    }
}

impl Default for ValueSchema {
    fn default() -> Self {
        ValueSchema::new(
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
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S.%fZ").to_string(),
            meta: Meta::new(MetaType::STATE),
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
    pub fn new(min: f64, max: f64, step: f64, unit: &str) -> Self {
        NumberSchema {
            min,
            max,
            step,
            unit: unit.to_string(),
        }
    }
}

impl Default for NumberSchema {
    fn default() -> Self {
        NumberSchema::new(0f64, 1f64, 1f64, "")
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
    pub meta_type: MetaType,
    pub version: String,
}

impl Meta {
    pub fn new_with_uuid(id: Uuid, meta_type: MetaType) -> Self {
        Meta {
            id,
            meta_type,
            version: String::from("2.0"),
        }
    }

    pub fn new(meta_type: MetaType) -> Self {
        Meta {
            id: Uuid::new_v4(),
            meta_type,
            version: String::from("2.0"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MetaType {
    NETWORK,
    DEVICE,
    VALUE,
    STATE,
}
