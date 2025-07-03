use crate::messages::{P2PMessage, GossipTopic};
use common::{Result, Error};
use libp2p::{
    core::upgrade,
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identify,
    kad::{self, store::MemoryStore},
    mdns,
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
    PeerId, Swarm, Transport,
};
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

/// Network behavior combining multiple protocols
#[derive(NetworkBehaviour)]
pub struct XpNetBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

/// P2P network node
pub struct NetworkNode {
    swarm: Swarm<XpNetBehaviour>,
    peer_id: PeerId,
    message_tx: mpsc::Sender<NetworkEvent>,
    message_rx: mpsc::Receiver<NetworkCommand>,
}

/// Network events emitted to application
#[derive(Debug)]
pub enum NetworkEvent {
    /// Received a message from a peer
    MessageReceived { peer: PeerId, message: P2PMessage },
    
    /// New peer connected
    PeerConnected { peer: PeerId },
    
    /// Peer disconnected
    PeerDisconnected { peer: PeerId },
    
    /// Discovered new peers via mDNS
    PeersDiscovered { peers: Vec<PeerId> },
}

/// Commands to control the network
#[derive(Debug)]
pub enum NetworkCommand {
    /// Send message to specific peer
    SendMessage { peer: PeerId, message: P2PMessage },
    
    /// Broadcast message via gossipsub
    Broadcast { topic: GossipTopic, message: P2PMessage },
    
    /// Connect to a peer
    ConnectPeer { peer: PeerId, addr: String },
    
    /// Subscribe to gossip topic
    Subscribe { topic: GossipTopic },
}

impl NetworkNode {
    pub async fn new(
        listen_addr: &str,
    ) -> Result<(Self, mpsc::Receiver<NetworkEvent>, mpsc::Sender<NetworkCommand>)> {
        // Create identity
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        info!("Local peer ID: {}", peer_id);
        
        // Create transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();
        
        // Create Gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .duplicate_cache_time(Duration::from_secs(60))
            .build()
            .map_err(|e| Error::Network(format!("Invalid gossipsub config: {}", e)))?;
        
        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| Error::Network(format!("Failed to create gossipsub: {}", e)))?;
        
        // Create Kademlia
        let kademlia = kad::Behaviour::new(peer_id, MemoryStore::new(peer_id));
        
        // Create mDNS for local peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
            .map_err(|e| Error::Network(format!("Failed to create mDNS: {}", e)))?;
        
        // Create identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/xpnet/1.0.0".to_string(),
            local_key.public(),
        ));
        
        // Combine behaviors
        let behaviour = XpNetBehaviour {
            gossipsub,
            kademlia,
            mdns,
            identify,
        };
        
        // Create swarm
        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        
        // Listen on the specified address
        swarm.listen_on(listen_addr.parse().map_err(|_| Error::Network("Invalid listen address".into()))?)
            .map_err(|e| Error::Network(format!("Failed to listen: {}", e)))?;
        
        // Create channels
        let (event_tx, event_rx) = mpsc::channel(1000);
        let (cmd_tx, cmd_rx) = mpsc::channel(1000);
        
        let node = Self {
            swarm,
            peer_id,
            message_tx: event_tx,
            message_rx: cmd_rx,
        };
        
        Ok((node, event_rx, cmd_tx))
    }
    
    pub async fn run(mut self) {
        // Subscribe to default topics
        for topic in [
            GossipTopic::Transactions,
            GossipTopic::Validations,
            GossipTopic::Tips,
            GossipTopic::StateUpdates,
        ] {
            let ident_topic = IdentTopic::new(topic.as_str());
            if let Err(e) = self.swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                error!("Failed to subscribe to topic {:?}: {}", topic, e);
            }
        }
        
        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(behaviour_event) => {
                            self.handle_behaviour_event(behaviour_event).await;
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            info!("Connected to peer: {}", peer_id);
                            let _ = self.message_tx.send(NetworkEvent::PeerConnected { peer: peer_id }).await;
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            info!("Disconnected from peer: {}", peer_id);
                            let _ = self.message_tx.send(NetworkEvent::PeerDisconnected { peer: peer_id }).await;
                        }
                        _ => {}
                    }
                }
                
                // Handle commands
                Some(cmd) = self.message_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }
    
    async fn handle_behaviour_event(&mut self, event: XpNetBehaviourEvent) {
        match event {
            XpNetBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            }) => {
                debug!("Received gossip message from {}", propagation_source);
                
                // Deserialize message
                match serde_json::from_slice::<P2PMessage>(&message.data) {
                    Ok(msg) => {
                        let _ = self.message_tx.send(NetworkEvent::MessageReceived {
                            peer: propagation_source,
                            message: msg,
                        }).await;
                    }
                    Err(e) => {
                        warn!("Failed to deserialize message: {}", e);
                    }
                }
            }
            
            XpNetBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                let discovered: Vec<PeerId> = peers.map(|(peer, _)| peer).collect();
                info!("Discovered {} peers via mDNS", discovered.len());
                
                // Add to Kademlia
                for peer in &discovered {
                    self.swarm.behaviour_mut().kademlia.add_address(
                        peer,
                        "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
                    );
                }
                
                let _ = self.message_tx.send(NetworkEvent::PeersDiscovered { peers: discovered }).await;
            }
            
            XpNetBehaviourEvent::Identify(identify::Event::Received { peer_id, info }) => {
                debug!("Identified peer {}: {:?}", peer_id, info.protocol_version);
                
                // Add addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            
            _ => {}
        }
    }
    
    async fn handle_command(&mut self, cmd: NetworkCommand) {
        match cmd {
            NetworkCommand::Broadcast { topic, message } => {
                let data = serde_json::to_vec(&message).unwrap();
                let ident_topic = IdentTopic::new(topic.as_str());
                
                if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(ident_topic, data) {
                    error!("Failed to broadcast message: {}", e);
                }
            }
            
            NetworkCommand::SendMessage { peer, message } => {
                // For direct messages, would implement a request-response protocol
                warn!("Direct messaging not yet implemented");
            }
            
            NetworkCommand::ConnectPeer { peer, addr } => {
                match addr.parse() {
                    Ok(multiaddr) => {
                        self.swarm.behaviour_mut().kademlia.add_address(&peer, multiaddr);
                        // Trigger connection via Kademlia lookup
                        self.swarm.behaviour_mut().kademlia.get_closest_peers(peer);
                    }
                    Err(e) => {
                        error!("Invalid address {}: {}", addr, e);
                    }
                }
            }
            
            NetworkCommand::Subscribe { topic } => {
                let ident_topic = IdentTopic::new(topic.as_str());
                if let Err(e) = self.swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                    error!("Failed to subscribe to topic {:?}: {}", topic, e);
                }
            }
        }
    }
    
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
}