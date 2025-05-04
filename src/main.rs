mod crds;

use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, PersistentVolumeClaimSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta};
use kube::{Api, Client, ResourceExt, runtime::controller::{Action, Controller}, Resource};
use kube::api::{Patch, PatchParams};
use crds::Dataset;

#[derive(Clone)]
struct Context {
    pub client: Client,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("owner reference could not be created")]
    OwnerReferenceFailed,
    #[error(transparent)]
    KubeError(#[from] kube::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn create_owned_pvc(obj: &Dataset) -> Result<PersistentVolumeClaim> {
    let owner_ref = obj.controller_owner_ref(&()).ok_or(Error::OwnerReferenceFailed)?;
    Ok(PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(obj.spec.name.clone()),
            owner_references: Some(vec![owner_ref]),
            ..ObjectMeta::default()
        },
        spec: Some(PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".into()]),
            ..obj.spec.storage.clone().unwrap_or_default()
        })
        ,
        ..Default::default()
    })
}

async fn reconcile(obj: Arc<Dataset>, ctx: Arc<Context>) -> Result<Action> {
    println!("reconcile request: {}", obj.name_any());
    // Request head for dataset url to get size of file
    

    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), obj.namespace().unwrap_or("default".to_string()).as_str());

    let pvc_data = create_owned_pvc(&obj)?;
    println!("pvc: {:?}", pvc_data);
    
    let patch_params = PatchParams::apply("dataset-controller");
    let patch = Patch::Apply(pvc_data.clone());
    let _pvc = pvcs.patch(pvc_data.name_any().as_str(), &patch_params, &patch).await?;

    println!("pvc: {:?}", _pvc);
    
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