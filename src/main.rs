mod crds;

use crate::crds::DatasetStatus;
use crds::Dataset;
use futures::StreamExt;
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, PersistentVolumeClaimSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::chrono;
use kube::api::{Patch, PatchParams};
use kube::{
    Api, Client, Resource, ResourceExt,
    runtime::controller::{Action, Controller},
};
use serde_json::json;
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
struct Context {
    pub client: Client,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("owner reference could not be created")]
    OwnerReferenceFailed,
    #[error("unknown status")]
    UnknownStatus,
    #[error(transparent)]
    KubeError(#[from] kube::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn create_owned_pvc(obj: &Dataset) -> Result<PersistentVolumeClaim> {
    let owner_ref = obj
        .controller_owner_ref(&())
        .ok_or(Error::OwnerReferenceFailed)?;
    Ok(PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(obj.spec.name.clone()),
            owner_references: Some(vec![owner_ref]),
            ..ObjectMeta::default()
        },
        spec: Some(PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".into()]),
            ..obj.spec.storage.clone().unwrap_or_default()
        }),
        ..Default::default()
    })
}

async fn update_status(obj: &Dataset, api: &Api<Dataset>, phase: &str) -> Result<()> {
    let status = json!({
        "status": DatasetStatus {
            phase: phase.into(),
            last_updated: Some(chrono::Utc::now()),
        }   
    });
    api.patch_status(
        obj.name_any().as_str(),
        &PatchParams::default(),
        &Patch::Merge(&status),
    )
    .await?;
    Ok(())
}

async fn patch_pvc(
    pvc_data: PersistentVolumeClaim,
    api: &Api<PersistentVolumeClaim>,
) -> Result<()> {
    let patch_params = PatchParams::apply("dataset-controller");
    let patch = Patch::Apply(pvc_data.clone());
    api.patch(pvc_data.name_any().as_str(), &patch_params, &patch)
        .await?;
    Ok(())
}

async fn reconcile(obj: Arc<Dataset>, ctx: Arc<Context>) -> Result<Action> {
    println!("reconcile request: {}", obj.name_any());
    let api: Api<Dataset> = Api::namespaced(
        ctx.client.clone(),
        obj.namespace().unwrap_or("default".to_string()).as_str(),
    );

    if obj.status.is_none() {
        update_status(&obj,&api,"Initializing").await?;
        return Ok(Action::requeue(Duration::from_secs(5)));
    }

    match obj.status.as_ref().unwrap().phase.as_str() {
        "Initializing" => {
            let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(
                ctx.client.clone(),
                obj.namespace().unwrap_or("default".to_string()).as_str(),
            );

            let pvc_data = create_owned_pvc(&obj)?;
            patch_pvc(pvc_data, &pvcs).await?;

            update_status(&obj, &api,"PVC Created").await?;
        }
        "PVC Created" => {
            println!("PVC already exists");
        }
        _ => return Err(Error::UnknownStatus),
    }

    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Dataset>, err: &Error, _ctx: Arc<Context>) -> Action {
    println!("Reconciliation error: {:?}", err);
    Action::requeue(Duration::from_secs(5))
}

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let client = Client::try_default().await?;

    let context = Arc::new(Context {
        client: client.clone(),
    });

    let datasets = Api::<Dataset>::all(client);

    Controller::new(datasets.clone(), Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}
