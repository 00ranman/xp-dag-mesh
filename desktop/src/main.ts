import { invoke } from "@tauri-apps/api/core";

// DOM references
const statusDot = document.getElementById("status-dot")!;
const statusText = document.getElementById("status-text")!;
const tierBadge = document.getElementById("tier-badge")!;
const peerCount = document.getElementById("peer-count")!;
const vertexCount = document.getElementById("vertex-count")!;
const btnStart = document.getElementById("btn-start")! as HTMLButtonElement;
const kellyEl = document.getElementById("kelly")!;
const sharpeEl = document.getElementById("sharpe")!;
const uptimeEl = document.getElementById("uptime")!;

// Sidebar navigation
document.querySelectorAll(".sidebar nav a").forEach((link) => {
  link.addEventListener("click", (e) => {
    e.preventDefault();
    document.querySelectorAll(".sidebar nav a").forEach((l) => l.classList.remove("active"));
    (e.target as HTMLElement).classList.add("active");
  });
});

// Node start/stop
let nodeRunning = false;

btnStart.addEventListener("click", async () => {
  if (!nodeRunning) {
    try {
      btnStart.textContent = "Starting...";
      btnStart.disabled = true;
      await invoke("start_mesh_node", { role: "full", bootstrapNodes: [] });
      nodeRunning = true;
      btnStart.textContent = "Stop Node";
      btnStart.disabled = false;
      statusDot.classList.add("connected");
      statusText.textContent = "Connected";
    } catch (err) {
      console.error("Failed to start node:", err);
      btnStart.textContent = "Start Node";
      btnStart.disabled = false;
    }
  } else {
    try {
      await invoke("stop_mesh_node");
      nodeRunning = false;
      btnStart.textContent = "Start Node";
      statusDot.classList.remove("connected");
      statusText.textContent = "Disconnected";
    } catch (err) {
      console.error("Failed to stop node:", err);
    }
  }
});

// Polling for status updates
async function pollStatus() {
  if (!nodeRunning) return;
  try {
    const status: any = await invoke("get_node_status");
    peerCount.textContent = status.peer_count?.toString() ?? "0";
    vertexCount.textContent = status.vertex_count?.toString() ?? "0";
    uptimeEl.textContent = status.uptime ?? "--";
  } catch (_) {}

  try {
    const tier: any = await invoke("get_tier_status");
    const tierName = tier.tier?.toLowerCase() ?? "observer";
    tierBadge.className = `tier-badge tier-${tierName}`;
    tierBadge.textContent = tier.tier ?? "Observer";
    kellyEl.textContent = tier.kelly?.toFixed(3) ?? "--";
    sharpeEl.textContent = tier.sharpe?.toFixed(2) ?? "--";
  } catch (_) {}
}

setInterval(pollStatus, 3000);

console.log("XP DAG Mesh desktop client initialized");
