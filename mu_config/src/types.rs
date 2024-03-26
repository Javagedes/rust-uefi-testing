use serde::Serialize;
use toml::Value;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone)]
pub enum Architecture {
    Common,
    X64,
    Custom(String)
}

impl TryFrom<&Value> for Architecture {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => match s.to_lowercase().as_str() {
                "common" => Ok(Architecture::Common),
                "x64" => Ok(Architecture::X64),
                v => Ok(Architecture::Custom(v.to_string())),
            },
            _ => Err(format!("Architecture must be a string, got {:?}", value)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone)]
pub enum Module {
    Common,
    Std,
    DxeDriver,
    Custom(String),
}

impl TryFrom<&Value> for Module {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => match s.to_lowercase().as_str() {
                "common" => Ok(Module::Common),
                "std" => Ok(Module::Std),
                "dxe_driver" => Ok(Module::DxeDriver),
                v => Ok(Module::Custom(v.to_string())),
            },
            _ => Err(format!("Module must be a string, got {:?}", value)),
        }
    }
}