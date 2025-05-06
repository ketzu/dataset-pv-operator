use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
use k8s_openapi::chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::immutability::immutable_string;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[kube(kind = "Dataset", version = "v1", group = "ketzu.net", namespaced)]
#[kube(status = "DatasetStatus")]
pub struct DatasetSpec {
    #[schemars(schema_with = "immutable_string")]
    pub name: String,
    #[schemars(schema_with = "immutable_string")]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<PersistentVolumeClaimSpec>
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DatasetStatus {
    pub phase: String,
    pub last_updated: Option<DateTime<Utc>>,
}