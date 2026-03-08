#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod node;
mod wallet;
mod validator;
mod sync;

use tauri::Manager;

/// XP DAG Mesh Desktop Client
/// 
/// Tauri backend that wraps the core mesh crates (dag_core, p2p_sync, validator_mesh)
/// into a desktop application with setup wizard, node management, and XP wallet.

#[tauri::command]
async fn generate_keypair() -> Result<String, String> {
    wallet::generate_ed25519_keypair()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_node_status() -> Result<node::NodeStatus, String> {
    node::get_status()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_mesh_node(role: String, bootstrap_nodes: Vec<String>) -> Result<(), String> {
    let node_role = match role.as_str() {
        "light" => node::NodeRole::Light,
        "validator" => node::NodeRole::Validator,
        "full" => node::NodeRole::Full,
        _ => return Err("Invalid role. Use: light, validator, full".into()),
    };
    node::start(node_role, bootstrap_nodes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_mesh_node() -> Result<(), String> {
    node::stop()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_xp_balance() -> Result<f64, String> {
    wallet::get_balance()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn submit_entropy_claim(domain: String, delta_s: f64) -> Result<String, String> {
    validator::submit_claim(domain, delta_s)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_mesh_topology() -> Result<Vec<node::PeerInfo>, String> {
    sync::get_connected_peers()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tier_status() -> Result<validator::TierStatus, String> {
    validator::get_tier()
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_keypair,
            get_node_status,
            start_mesh_node,
            stop_mesh_node,
            get_xp_balance,
            submit_entropy_claim,
            get_mesh_topology,
            get_tier_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running XP DAG Mesh desktop client");
}
