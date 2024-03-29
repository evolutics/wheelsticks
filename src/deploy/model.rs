use std::collections;

pub type ActualContainers = collections::BTreeSet<ActualContainer>;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct ActualContainer {
    pub container_id: String,
    pub service_config_hash: String,
    pub service_name: String,
}

pub type DesiredServices = collections::BTreeMap<String, DesiredServiceDefinition>;

pub struct DesiredServiceDefinition {
    pub replica_count: u16,
    pub service_config_hash: String,
    pub update_order: OperationOrder,
}

pub enum OperationOrder {
    StartFirst,
    StopFirst,
}

#[derive(Debug, PartialEq)]
pub enum ServiceContainerChange {
    Add {
        service_config_hash: String,
        service_name: String,
    },
    Keep {
        container_id: String,
        service_config_hash: String,
        service_name: String,
    },
    Remove {
        container_id: String,
        service_config_hash: String,
        service_name: String,
    },
}
