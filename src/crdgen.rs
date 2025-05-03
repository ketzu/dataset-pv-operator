use kube::CustomResourceExt;

mod crds;
fn main() {
    print!("{}", serde_yaml::to_string(&crds::Dataset::crd()).unwrap())
}