use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}
