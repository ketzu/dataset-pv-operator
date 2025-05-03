use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
use k8s_openapi::chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[kube(kind = "Dataset", version = "v1", group = "ketzu.net")]
#[kube(status = "DatasetStatus")]
pub struct DatasetSpec {
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<PersistentVolumeClaimSpec>
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DatasetStatus {
    phase: String,
    last_updated: Option<DateTime<Utc>>,
}