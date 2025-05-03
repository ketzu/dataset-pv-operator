mod crds;

use std::{sync::Arc, time::Duration};
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client, ResourceExt,
    runtime::controller::{Action, Controller}
};

use crds::Dataset;

#[derive(thiserror::Error, Debug)]
pub enum Error {}
pub type Result<T, E = Error> = std::result::Result<T, E>;

async fn reconcile(obj: Arc<Dataset>, ctx: Arc<()>) -> Result<Action> {
    println!("reconcile request: {}", obj.name_any());
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<Dataset>, _err: &Error, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let client = Client::try_default().await?;
    let datasets = Api::<Dataset>::all(client);

    Controller::new(datasets.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(()))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}