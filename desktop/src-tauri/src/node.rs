//! Mesh node lifecycle management
//!
//! Wraps dag_core and p2p_sync into a managed node that can be
//! started, stopped, and queried from the Tauri frontend.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static NODE_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRole {
    Light,      // Relay only, minimal DAG state
    Validator,  // Entropy verification + consensus
    Full,       // Complete DAG archive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub running: bool,
    pub role: Option<NodeRole>,
    pub peer_count: usize,
    pub dag_height: u64,
    pub sync_progress: f64, // 0.0 - 1.0
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub role: NodeRole,
    pub latency_ms: u32,
    pub reputation: f64,
}

pub fn get_status() -> Result<NodeStatus, Box<dyn std::error::Error>> {
    Ok(NodeStatus {
        running: NODE_RUNNING.load(Ordering::Relaxed),
        role: None,
        peer_count: 0,
        dag_height: 0,
        sync_progress: 0.0,
        uptime_seconds: 0,
    })
}

pub async fn start(
    role: NodeRole,
    bootstrap_nodes: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if NODE_RUNNING.load(Ordering::Relaxed) {
        return Err("Node is already running".into());
    }

    // TODO: Initialize dag_core::Dag
    // TODO: Initialize p2p_sync with bootstrap nodes
    // TODO: Start validator_mesh if role == Validator
    // TODO: Begin DAG sync state machine

    NODE_RUNNING.store(true, Ordering::Relaxed);
    log::info!("Mesh node started as {:?} with {} bootstrap nodes", role, bootstrap_nodes.len());
    Ok(())
}

pub async fn stop() -> Result<(), Box<dyn std::error::Error>> {
    if !NODE_RUNNING.load(Ordering::Relaxed) {
        return Err("Node is not running".into());
    }

    // TODO: Graceful shutdown of p2p connections
    // TODO: Flush DAG state to disk
    // TODO: Stop validator mesh

    NODE_RUNNING.store(false, Ordering::Relaxed);
    log::info!("Mesh node stopped");
    Ok(())
}
