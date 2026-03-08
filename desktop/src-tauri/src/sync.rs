//! Mesh network synchronization protocol
//! Handles peer discovery, DAG sync, and state reconciliation

use crate::node::{MeshNode, NodeRole};
use crate::validator::DagVertex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Sync protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Announce presence to mesh
    Hello {
        node_id: String,
        role: NodeRole,
        dag_tip: String,
        vertex_count: u64,
    },
    /// Request missing vertices by ID
    RequestVertices { ids: Vec<String> },
    /// Respond with requested vertices
    VertexBatch { vertices: Vec<DagVertex> },
    /// Announce new vertex to peers
    NewVertex(DagVertex),
    /// Request peer list
    GetPeers,
    /// Respond with known peers
    PeerList { peers: Vec<PeerInfo> },
    /// Heartbeat for liveness
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}

/// Peer connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub address: String,
    pub port: u16,
    pub role: NodeRole,
    pub last_seen: u64,
}

/// Sync state machine
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    Disconnected,
    Discovering,
    Syncing { progress: f64 },
    Synchronized,
}

/// Manages mesh synchronization
pub struct SyncManager {
    pub state: SyncState,
    pub peers: HashMap<String, PeerInfo>,
    pub known_vertex_ids: HashSet<String>,
    pub pending_requests: Vec<String>,
    pub max_peers: usize,
    pub sync_batch_size: usize,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self {
            state: SyncState::Disconnected,
            peers: HashMap::new(),
            known_vertex_ids: HashSet::new(),
            pending_requests: Vec::new(),
            max_peers: 50,
            sync_batch_size: 100,
        }
    }
}

impl SyncManager {
    /// Create hello message for this node
    pub fn create_hello(&self, node: &MeshNode) -> SyncMessage {
        SyncMessage::Hello {
            node_id: node.id.clone(),
            role: node.role.clone(),
            dag_tip: self
                .known_vertex_ids
                .iter()
                .last()
                .cloned()
                .unwrap_or_default(),
            vertex_count: self.known_vertex_ids.len() as u64,
        }
    }

    /// Process incoming sync message
    pub fn handle_message(
        &mut self,
        from: &str,
        msg: SyncMessage,
    ) -> Option<SyncMessage> {
        match msg {
            SyncMessage::Hello {
                node_id,
                role,
                dag_tip,
                vertex_count,
            } => {
                self.peers.insert(
                    node_id.clone(),
                    PeerInfo {
                        node_id,
                        address: from.to_string(),
                        port: 0,
                        role,
                        last_seen: 0,
                    },
                );
                if vertex_count > self.known_vertex_ids.len() as u64 {
                    self.state = SyncState::Syncing { progress: 0.0 };
                }
                None
            }
            SyncMessage::RequestVertices { ids } => {
                // Would return matching vertices from local DAG
                Some(SyncMessage::VertexBatch {
                    vertices: Vec::new(),
                })
            }
            SyncMessage::VertexBatch { vertices } => {
                for v in &vertices {
                    self.known_vertex_ids.insert(v.id.clone());
                }
                self.update_sync_progress();
                None
            }
            SyncMessage::NewVertex(vertex) => {
                self.known_vertex_ids.insert(vertex.id.clone());
                None
            }
            SyncMessage::GetPeers => Some(SyncMessage::PeerList {
                peers: self.peers.values().cloned().collect(),
            }),
            SyncMessage::Ping { timestamp } => {
                Some(SyncMessage::Pong { timestamp })
            }
            _ => None,
        }
    }

    /// Update sync progress estimate
    fn update_sync_progress(&mut self) {
        if let SyncState::Syncing { .. } = self.state {
            if self.pending_requests.is_empty() {
                self.state = SyncState::Synchronized;
            }
        }
    }

    /// Get IDs we need from a peer
    pub fn missing_from(&self, peer_ids: &[String]) -> Vec<String> {
        peer_ids
            .iter()
            .filter(|id| !self.known_vertex_ids.contains(*id))
            .cloned()
            .collect()
    }
}
