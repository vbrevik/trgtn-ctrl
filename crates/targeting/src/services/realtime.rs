// Real-time Updates Service
// Broadcasts targeting-related updates via SSE/WebSocket

use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use chrono::Utc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TargetingUpdate {
    pub r#type: String, // target_status_changed, bda_assessment_created, etc.
    pub entity_id: String,
    pub entity_type: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct RealtimeService {
    tx: broadcast::Sender<TargetingUpdate>,
}

impl RealtimeService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TargetingUpdate> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, update: TargetingUpdate) -> Result<(), broadcast::error::SendError<TargetingUpdate>> {
        self.tx.send(update).map(|_| ())
    }

    // Helper methods for common update types
    pub fn broadcast_target_status_changed(&self, target_id: &str, old_status: &str, new_status: &str) {
        let update = TargetingUpdate {
            r#type: "target_status_changed".to_string(),
            entity_id: target_id.to_string(),
            entity_type: "target".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "old_status": old_status,
                "new_status": new_status,
            }),
        };
        let _ = self.broadcast(update);
    }

    pub fn broadcast_bda_created(&self, bda_id: &str, target_id: &str) {
        let update = TargetingUpdate {
            r#type: "bda_assessment_created".to_string(),
            entity_id: bda_id.to_string(),
            entity_type: "bda".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "target_id": target_id,
            }),
        };
        let _ = self.broadcast(update);
    }

    pub fn broadcast_decision_gate_changed(&self, gate_id: &str, status: &str) {
        let update = TargetingUpdate {
            r#type: "decision_gate_changed".to_string(),
            entity_id: gate_id.to_string(),
            entity_type: "decision_gate".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "status": status,
            }),
        };
        let _ = self.broadcast(update);
    }

    pub fn broadcast_jtb_session_updated(&self, session_id: &str, action: &str) {
        let update = TargetingUpdate {
            r#type: "jtb_session_updated".to_string(),
            entity_id: session_id.to_string(),
            entity_type: "jtb_session".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "action": action,
            }),
        };
        let _ = self.broadcast(update);
    }

    pub fn broadcast_tst_alert(&self, target_id: &str, urgency: &str) {
        let update = TargetingUpdate {
            r#type: "tst_alert".to_string(),
            entity_id: target_id.to_string(),
            entity_type: "target".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "urgency": urgency,
            }),
        };
        let _ = self.broadcast(update);
    }

    pub fn broadcast_new_target_nominated(&self, target_id: &str, name: &str) {
        let update = TargetingUpdate {
            r#type: "new_target_nominated".to_string(),
            entity_id: target_id.to_string(),
            entity_type: "target".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "name": name,
            }),
        };
        let _ = self.broadcast(update);
    }
}

impl Default for RealtimeService {
    fn default() -> Self {
        Self::new()
    }
}
