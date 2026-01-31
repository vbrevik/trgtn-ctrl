// Tests for Real-time Updates Endpoints

#[cfg(test)]
mod tests {
    use crate::features::targeting::services::realtime::RealtimeService;

    #[tokio::test]
    async fn test_realtime_service_broadcast() {
        let service = RealtimeService::new();
        
        // Subscribe before broadcasting
        let mut rx = service.subscribe();
        
        // Broadcast an update
        service.broadcast_target_status_changed("target-1", "NOMINATED", "VALIDATED");
        
        // Should receive the update
        let update = rx.recv().await.unwrap();
        assert_eq!(update.r#type, "target_status_changed");
        assert_eq!(update.entity_id, "target-1");
        assert_eq!(update.entity_type, "target");
    }

    #[tokio::test]
    async fn test_realtime_service_multiple_subscribers() {
        let service = RealtimeService::new();
        
        let mut rx1 = service.subscribe();
        let mut rx2 = service.subscribe();
        
        service.broadcast_new_target_nominated("target-2", "Test Target");
        
        // Both should receive
        let update1 = rx1.recv().await.unwrap();
        let update2 = rx2.recv().await.unwrap();
        
        assert_eq!(update1.entity_id, update2.entity_id);
        assert_eq!(update1.r#type, "new_target_nominated");
    }

    #[tokio::test]
    async fn test_realtime_service_helper_methods() {
        let service = RealtimeService::new();
        let mut rx = service.subscribe();
        
        // Test all helper methods
        service.broadcast_bda_created("bda-1", "target-1");
        let update = rx.recv().await.unwrap();
        assert_eq!(update.r#type, "bda_assessment_created");
        
        service.broadcast_decision_gate_changed("gate-1", "GO");
        let update = rx.recv().await.unwrap();
        assert_eq!(update.r#type, "decision_gate_changed");
        
        service.broadcast_jtb_session_updated("session-1", "target_added");
        let update = rx.recv().await.unwrap();
        assert_eq!(update.r#type, "jtb_session_updated");
        
        service.broadcast_tst_alert("target-3", "CRITICAL");
        let update = rx.recv().await.unwrap();
        assert_eq!(update.r#type, "tst_alert");
    }
}
