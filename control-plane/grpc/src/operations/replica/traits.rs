use crate::{
    common,
    grpc_opts::Context,
    replica,
    replica::{
        get_replicas_request, CreateReplicaRequest, DestroyReplicaRequest, ShareReplicaRequest,
        UnshareReplicaRequest,
    },
};
use common_lib::{
    mbus_api::{v0::Replicas, ReplyError, ResourceKind},
    types::v0::{
        message_bus,
        message_bus::{
            CreateReplica, DestroyReplica, Filter, NexusId, NodeId, PoolId, Replica, ReplicaId,
            ReplicaName, ReplicaOwners, ShareReplica, UnshareReplica, VolumeId,
        },
    },
};
use std::convert::TryFrom;

/// all replica operations to be a part of the ReplicaOperations trait
#[tonic::async_trait]
pub trait ReplicaOperations: Send + Sync {
    async fn create(
        &self,
        req: &dyn CreateReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<Replica, ReplyError>;
    async fn get(&self, filter: Filter, ctx: Option<Context>) -> Result<Replicas, ReplyError>;
    async fn destroy(
        &self,
        req: &dyn DestroyReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<(), ReplyError>;
    async fn share(
        &self,
        req: &dyn ShareReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<String, ReplyError>;
    async fn unshare(
        &self,
        req: &dyn UnshareReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<(), ReplyError>;
}

impl From<Replica> for replica::Replica {
    fn from(replica: Replica) -> Self {
        let share: common::Protocol = replica.share.into();
        let status: replica::ReplicaStatus = replica.status.into();
        replica::Replica {
            node_id: replica.node.into(),
            name: replica.name.into(),
            replica_id: Some(replica.uuid.into()),
            pool_id: replica.pool.into(),
            thin: replica.thin,
            size: replica.size,
            share: share as i32,
            uri: replica.uri,
            status: status as i32,
        }
    }
}

impl TryFrom<replica::Replica> for Replica {
    type Error = ReplyError;
    fn try_from(replica: replica::Replica) -> Result<Self, Self::Error> {
        Ok(Replica {
            node: replica.node_id.into(),
            name: replica.name.into(),
            uuid: match replica.replica_id {
                Some(string) => match ReplicaId::try_from(string) {
                    Ok(replica_id) => replica_id,
                    Err(_) => return Err(ReplyError::unwrap_err(ResourceKind::Replica)),
                },
                None => return Err(ReplyError::unwrap_err(ResourceKind::Replica)),
            },
            pool: replica.pool_id.into(),
            thin: replica.thin,
            size: replica.size,
            share: match common::Protocol::from_i32(replica.share) {
                Some(share) => share.into(),
                None => return Err(ReplyError::unwrap_err(ResourceKind::Replica)),
            },
            uri: replica.uri,
            status: match replica::ReplicaStatus::from_i32(replica.status) {
                Some(status) => status.into(),
                None => return Err(ReplyError::unwrap_err(ResourceKind::Replica)),
            },
        })
    }
}

impl From<get_replicas_request::Filter> for Filter {
    fn from(filter: get_replicas_request::Filter) -> Self {
        match filter {
            get_replicas_request::Filter::Node(node_filter) => {
                Filter::Node(node_filter.node_id.into())
            }
            get_replicas_request::Filter::NodePool(node_pool_filter) => Filter::NodePool(
                node_pool_filter.node_id.into(),
                node_pool_filter.pool_id.into(),
            ),
            get_replicas_request::Filter::Pool(pool_filter) => {
                Filter::Pool(pool_filter.pool_id.into())
            }
            get_replicas_request::Filter::NodePoolReplica(node_pool_replica_filter) => {
                Filter::NodePoolReplica(
                    node_pool_replica_filter.node_id.into(),
                    node_pool_replica_filter.pool_id.into(),
                    ReplicaId::try_from(node_pool_replica_filter.replica_id).unwrap(),
                )
            }
            get_replicas_request::Filter::NodeReplica(node_replica_filter) => Filter::NodeReplica(
                node_replica_filter.node_id.into(),
                ReplicaId::try_from(node_replica_filter.replica_id).unwrap(),
            ),
            get_replicas_request::Filter::PoolReplica(pool_replica_filter) => Filter::PoolReplica(
                pool_replica_filter.pool_id.into(),
                ReplicaId::try_from(pool_replica_filter.replica_id).unwrap(),
            ),
            get_replicas_request::Filter::Replica(replica_filter) => {
                Filter::Replica(ReplicaId::try_from(replica_filter.replica_id).unwrap())
            }
            get_replicas_request::Filter::Volume(volume_filter) => {
                Filter::Volume(VolumeId::try_from(volume_filter.volume_id).unwrap())
            }
        }
    }
}

impl TryFrom<replica::Replicas> for Replicas {
    type Error = ReplyError;
    fn try_from(grpc_replicas_type: replica::Replicas) -> Result<Self, Self::Error> {
        let mut replicas: Vec<Replica> = vec![];
        for replica in grpc_replicas_type.replicas {
            replicas.push(Replica::try_from(replica.clone())?)
        }
        Ok(Replicas(replicas))
    }
}

impl From<Replicas> for replica::Replicas {
    fn from(replicas: Replicas) -> Self {
        replica::Replicas {
            replicas: replicas
                .into_inner()
                .iter()
                .map(|replicas| replicas.clone().into())
                .collect(),
        }
    }
}

/// CreateReplicaInfo trait for the replica creation to be implemented by entities which want to
/// avail this operation
pub trait CreateReplicaInfo: Send + Sync {
    fn node(&self) -> NodeId;
    fn name(&self) -> Option<ReplicaName>;
    fn uuid(&self) -> ReplicaId;
    fn pool(&self) -> PoolId;
    fn size(&self) -> u64;
    fn thin(&self) -> bool;
    fn share(&self) -> message_bus::Protocol;
    fn managed(&self) -> bool;
    fn owners(&self) -> ReplicaOwners;
}

impl CreateReplicaInfo for CreateReplica {
    fn node(&self) -> NodeId {
        self.node.clone()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone()
    }

    fn uuid(&self) -> ReplicaId {
        self.uuid.clone()
    }

    fn pool(&self) -> PoolId {
        self.pool.clone()
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn thin(&self) -> bool {
        self.thin
    }

    fn share(&self) -> message_bus::Protocol {
        self.share
    }

    fn managed(&self) -> bool {
        self.managed
    }

    fn owners(&self) -> ReplicaOwners {
        self.owners.clone()
    }
}

impl CreateReplicaInfo for CreateReplicaRequest {
    fn node(&self) -> NodeId {
        self.node_id.clone().into()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone().map(|e| e.into())
    }

    fn uuid(&self) -> ReplicaId {
        ReplicaId::try_from(self.replica_id.clone().unwrap()).unwrap()
    }

    fn pool(&self) -> PoolId {
        self.pool_id.clone().into()
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn thin(&self) -> bool {
        self.thin
    }

    fn share(&self) -> message_bus::Protocol {
        common::Protocol::from_i32(self.share).unwrap().into()
    }

    fn managed(&self) -> bool {
        self.managed
    }

    fn owners(&self) -> ReplicaOwners {
        ReplicaOwners::new(
            self.owners
                .clone()
                .unwrap()
                .volume
                .map(|id| VolumeId::try_from(id).unwrap()),
            self.owners
                .clone()
                .unwrap()
                .nexuses
                .iter()
                .map(|id| NexusId::try_from(id.clone()).unwrap())
                .collect(),
        )
    }
}

/// DestroyReplicaInfo trait for the replica deletion to be implemented by entities which want to
/// avail this operation
pub trait DestroyReplicaInfo: Send + Sync {
    fn node(&self) -> NodeId;
    fn pool(&self) -> PoolId;
    fn name(&self) -> Option<ReplicaName>;
    fn uuid(&self) -> ReplicaId;
    fn disowners(&self) -> ReplicaOwners;
}

impl DestroyReplicaInfo for DestroyReplica {
    fn node(&self) -> NodeId {
        self.node.clone()
    }

    fn pool(&self) -> PoolId {
        self.pool.clone()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone()
    }

    fn uuid(&self) -> ReplicaId {
        self.uuid.clone()
    }

    fn disowners(&self) -> ReplicaOwners {
        self.disowners.clone()
    }
}

impl DestroyReplicaInfo for DestroyReplicaRequest {
    fn node(&self) -> NodeId {
        self.node_id.clone().into()
    }

    fn pool(&self) -> PoolId {
        self.pool_id.clone().into()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone().map(|e| e.into())
    }

    fn uuid(&self) -> ReplicaId {
        ReplicaId::try_from(self.replica_id.clone().unwrap()).unwrap()
    }

    fn disowners(&self) -> ReplicaOwners {
        ReplicaOwners::new(
            self.disowners
                .clone()
                .unwrap()
                .volume
                .map(|id| VolumeId::try_from(id).unwrap()),
            self.disowners
                .clone()
                .unwrap()
                .nexuses
                .iter()
                .map(|id| NexusId::try_from(id.clone()).unwrap())
                .collect(),
        )
    }
}

/// ShareReplicaInfo trait for the replica sharing to be implemented by entities which want to avail
/// this operation
pub trait ShareReplicaInfo: Send + Sync {
    fn node(&self) -> NodeId;
    fn pool(&self) -> PoolId;
    fn name(&self) -> Option<ReplicaName>;
    fn uuid(&self) -> ReplicaId;
    fn protocol(&self) -> message_bus::ReplicaShareProtocol;
}

impl ShareReplicaInfo for ShareReplica {
    fn node(&self) -> NodeId {
        self.node.clone()
    }

    fn pool(&self) -> PoolId {
        self.pool.clone()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone()
    }

    fn uuid(&self) -> ReplicaId {
        self.uuid.clone()
    }

    fn protocol(&self) -> message_bus::ReplicaShareProtocol {
        self.protocol
    }
}

impl ShareReplicaInfo for ShareReplicaRequest {
    fn node(&self) -> NodeId {
        self.node_id.clone().into()
    }

    fn pool(&self) -> PoolId {
        self.pool_id.clone().into()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone().map(|e| e.into())
    }

    fn uuid(&self) -> ReplicaId {
        ReplicaId::try_from(self.replica_id.clone().unwrap()).unwrap()
    }

    fn protocol(&self) -> message_bus::ReplicaShareProtocol {
        replica::ReplicaShareProtocol::from_i32(self.protocol)
            .unwrap()
            .into()
    }
}

/// UnshareReplicaInfo trait for the replica sharing to be implemented by entities which want to
/// avail this operation
pub trait UnshareReplicaInfo: Send + Sync {
    fn node(&self) -> NodeId;
    fn pool(&self) -> PoolId;
    fn name(&self) -> Option<ReplicaName>;
    fn uuid(&self) -> ReplicaId;
}

impl UnshareReplicaInfo for UnshareReplica {
    fn node(&self) -> NodeId {
        self.node.clone()
    }

    fn pool(&self) -> PoolId {
        self.pool.clone()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone()
    }

    fn uuid(&self) -> ReplicaId {
        self.uuid.clone()
    }
}

impl UnshareReplicaInfo for UnshareReplicaRequest {
    fn node(&self) -> NodeId {
        self.node_id.clone().into()
    }

    fn pool(&self) -> PoolId {
        self.pool_id.clone().into()
    }

    fn name(&self) -> Option<ReplicaName> {
        self.name.clone().map(|e| e.into())
    }

    fn uuid(&self) -> ReplicaId {
        ReplicaId::try_from(self.replica_id.clone().unwrap()).unwrap()
    }
}

impl From<&dyn CreateReplicaInfo> for CreateReplicaRequest {
    fn from(data: &dyn CreateReplicaInfo) -> Self {
        let share: common::Protocol = data.share().into();
        Self {
            node_id: data.node().to_string(),
            pool_id: data.pool().to_string(),
            name: data.name().map(|name| name.to_string()),
            replica_id: Some(data.uuid().to_string()),
            thin: data.thin(),
            size: data.size(),
            share: share as i32,
            managed: data.managed(),
            owners: Some(replica::ReplicaOwners {
                volume: data.owners().volume().map(|id| id.to_string()),
                nexuses: data
                    .owners()
                    .nexuses()
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
            }),
        }
    }
}

impl From<&dyn CreateReplicaInfo> for CreateReplica {
    fn from(data: &dyn CreateReplicaInfo) -> Self {
        Self {
            node: data.node(),
            name: data.name(),
            uuid: data.uuid(),
            pool: data.pool(),
            size: data.size(),
            thin: data.thin(),
            share: data.share(),
            managed: data.managed(),
            owners: data.owners(),
        }
    }
}

impl From<&dyn DestroyReplicaInfo> for DestroyReplicaRequest {
    fn from(data: &dyn DestroyReplicaInfo) -> Self {
        Self {
            node_id: data.node().to_string(),
            pool_id: data.pool().to_string(),
            name: data.name().map(|name| name.to_string()),
            replica_id: Some(data.uuid().to_string()),
            disowners: Some(replica::ReplicaOwners {
                volume: data.disowners().volume().map(|id| id.to_string()),
                nexuses: data
                    .disowners()
                    .nexuses()
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
            }),
        }
    }
}

impl From<&dyn DestroyReplicaInfo> for DestroyReplica {
    fn from(data: &dyn DestroyReplicaInfo) -> Self {
        Self {
            node: data.node(),
            pool: data.pool(),
            uuid: data.uuid(),
            name: data.name(),
            disowners: data.disowners(),
        }
    }
}

impl From<&dyn ShareReplicaInfo> for ShareReplicaRequest {
    fn from(data: &dyn ShareReplicaInfo) -> Self {
        let protocol: replica::ReplicaShareProtocol = data.protocol().into();
        Self {
            node_id: data.node().to_string(),
            pool_id: data.pool().to_string(),
            name: data.name().map(|name| name.to_string()),
            replica_id: Some(data.uuid().to_string()),
            protocol: protocol as i32,
        }
    }
}

impl From<&dyn ShareReplicaInfo> for ShareReplica {
    fn from(data: &dyn ShareReplicaInfo) -> Self {
        Self {
            node: data.node(),
            pool: data.pool(),
            uuid: data.uuid(),
            name: data.name(),
            protocol: data.protocol(),
        }
    }
}

impl From<&dyn UnshareReplicaInfo> for UnshareReplicaRequest {
    fn from(data: &dyn UnshareReplicaInfo) -> Self {
        Self {
            node_id: data.node().to_string(),
            pool_id: data.pool().to_string(),
            name: data.name().map(|name| name.to_string()),
            replica_id: Some(data.uuid().to_string()),
        }
    }
}

impl From<&dyn UnshareReplicaInfo> for UnshareReplica {
    fn from(data: &dyn UnshareReplicaInfo) -> Self {
        Self {
            node: data.node(),
            pool: data.pool(),
            uuid: data.uuid(),
            name: data.name(),
        }
    }
}

impl From<common::Protocol> for message_bus::Protocol {
    fn from(src: common::Protocol) -> Self {
        match src {
            common::Protocol::None => Self::None,
            common::Protocol::Nvmf => Self::Nvmf,
            common::Protocol::Iscsi => Self::Iscsi,
            common::Protocol::Nbd => Self::Nbd,
        }
    }
}

impl From<message_bus::Protocol> for common::Protocol {
    fn from(src: message_bus::Protocol) -> Self {
        match src {
            message_bus::Protocol::None => Self::None,
            message_bus::Protocol::Nvmf => Self::Nvmf,
            message_bus::Protocol::Iscsi => Self::Iscsi,
            message_bus::Protocol::Nbd => Self::Nbd,
        }
    }
}

impl From<replica::ReplicaStatus> for message_bus::ReplicaStatus {
    fn from(src: replica::ReplicaStatus) -> Self {
        match src {
            replica::ReplicaStatus::Unknown => Self::Unknown,
            replica::ReplicaStatus::Online => Self::Online,
            replica::ReplicaStatus::Degraded => Self::Degraded,
            replica::ReplicaStatus::Faulted => Self::Faulted,
        }
    }
}

impl From<message_bus::ReplicaStatus> for replica::ReplicaStatus {
    fn from(src: message_bus::ReplicaStatus) -> Self {
        match src {
            message_bus::ReplicaStatus::Unknown => Self::Unknown,
            message_bus::ReplicaStatus::Online => Self::Online,
            message_bus::ReplicaStatus::Degraded => Self::Degraded,
            message_bus::ReplicaStatus::Faulted => Self::Faulted,
        }
    }
}

impl From<replica::ReplicaShareProtocol> for message_bus::ReplicaShareProtocol {
    fn from(src: replica::ReplicaShareProtocol) -> Self {
        match src {
            replica::ReplicaShareProtocol::NvmfProtocol => Self::Nvmf,
        }
    }
}

impl From<message_bus::ReplicaShareProtocol> for replica::ReplicaShareProtocol {
    fn from(src: message_bus::ReplicaShareProtocol) -> Self {
        match src {
            message_bus::ReplicaShareProtocol::Nvmf => Self::NvmfProtocol,
        }
    }
}
