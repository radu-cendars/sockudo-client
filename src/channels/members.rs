//! Members tracking for presence channels.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Information about a channel member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    /// User ID
    pub user_id: String,
    /// Optional user info (arbitrary JSON)
    #[cfg(feature = "wasm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<Value>,
    /// Optional user info (JSON string for FFI)
    #[cfg(not(feature = "wasm"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<String>,
}

impl MemberInfo {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            user_info: None,
        }
    }

    #[cfg(feature = "wasm")]
    pub fn with_info(mut self, info: Value) -> Self {
        self.user_info = Some(info);
        self
    }

    #[cfg(not(feature = "wasm"))]
    pub fn with_info(mut self, info: String) -> Self {
        self.user_info = Some(info);
        self
    }

    pub fn with_info_value(mut self, info: Value) -> Self {
        #[cfg(feature = "wasm")]
        {
            self.user_info = Some(info);
        }
        #[cfg(not(feature = "wasm"))]
        {
            self.user_info = Some(info.to_string());
        }
        self
    }
}

/// Manages members of a presence channel
#[derive(Debug)]
pub struct Members {
    /// Map of user_id to member info
    members: RwLock<HashMap<String, MemberInfo>>,
    /// Current user's ID
    my_id: RwLock<Option<String>>,
}

impl Members {
    pub fn new() -> Self {
        Self {
            members: RwLock::new(HashMap::new()),
            my_id: RwLock::new(None),
        }
    }

    /// Set the current user's ID
    pub fn set_my_id(&self, id: impl Into<String>) {
        *self.my_id.write() = Some(id.into());
    }

    /// Get the current user's ID
    pub fn my_id(&self) -> Option<String> {
        self.my_id.read().clone()
    }

    /// Get the current user's member info
    pub fn me(&self) -> Option<MemberInfo> {
        let my_id = self.my_id.read();
        if let Some(ref id) = *my_id {
            self.members.read().get(id).cloned()
        } else {
            None
        }
    }

    /// Get a member by ID
    pub fn get(&self, user_id: &str) -> Option<MemberInfo> {
        self.members.read().get(user_id).cloned()
    }

    /// Get all members
    pub fn all(&self) -> Vec<MemberInfo> {
        self.members.read().values().cloned().collect()
    }

    /// Get member count
    pub fn count(&self) -> usize {
        self.members.read().len()
    }

    /// Add a member
    pub fn add(&self, member: MemberInfo) -> Option<MemberInfo> {
        let mut members = self.members.write();

        // Don't add if already exists
        if members.contains_key(&member.user_id) {
            return None;
        }

        members.insert(member.user_id.clone(), member.clone());
        Some(member)
    }

    /// Remove a member
    pub fn remove(&self, user_id: &str) -> Option<MemberInfo> {
        self.members.write().remove(user_id)
    }

    /// Initialize from subscription data
    pub fn on_subscription(&self, data: &Value) {
        let mut members = self.members.write();
        members.clear();

        if let Some(presence) = data.get("presence") {
            // Parse presence data
            if let Some(ids) = presence.get("ids").and_then(|v| v.as_array()) {
                let hash = presence.get("hash").and_then(|v| v.as_object());

                for id in ids {
                    if let Some(user_id) = id.as_str() {
                        #[cfg(feature = "wasm")]
                        let user_info = hash.and_then(|h| h.get(user_id)).cloned();
                        #[cfg(not(feature = "wasm"))]
                        let user_info = hash.and_then(|h| h.get(user_id)).map(|v| v.to_string());

                        let member = MemberInfo {
                            user_id: user_id.to_string(),
                            user_info,
                        };

                        members.insert(user_id.to_string(), member);
                    }
                }
            }
        }
    }

    /// Handle member added event
    pub fn add_member(&self, data: &Value) -> Option<MemberInfo> {
        let user_id = data.get("user_id")?.as_str()?;
        #[cfg(feature = "wasm")]
        let user_info = data.get("user_info").cloned();
        #[cfg(not(feature = "wasm"))]
        let user_info = data.get("user_info").map(|v| v.to_string());

        let member = MemberInfo {
            user_id: user_id.to_string(),
            user_info,
        };

        self.add(member)
    }

    /// Handle member removed event
    pub fn remove_member(&self, data: &Value) -> Option<MemberInfo> {
        let user_id = data.get("user_id")?.as_str()?;
        self.remove(user_id)
    }

    /// Reset members
    pub fn reset(&self) {
        self.members.write().clear();
        *self.my_id.write() = None;
    }

    /// Iterate over members
    pub fn each<F>(&self, mut f: F)
    where
        F: FnMut(&MemberInfo),
    {
        for member in self.members.read().values() {
            f(member);
        }
    }
}

impl Default for Members {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_members_add_remove() {
        let members = Members::new();

        let member = MemberInfo::new("user1");
        members.add(member.clone());

        assert_eq!(members.count(), 1);
        assert!(members.get("user1").is_some());

        members.remove("user1");
        assert_eq!(members.count(), 0);
    }

    #[test]
    fn test_my_id() {
        let members = Members::new();

        members.set_my_id("user1");
        #[cfg(feature = "wasm")]
        members.add(MemberInfo::new("user1").with_info(serde_json::json!({"name": "Test"})));
        #[cfg(not(feature = "wasm"))]
        members.add(
            MemberInfo::new("user1").with_info(serde_json::json!({"name": "Test"}).to_string()),
        );

        let me = members.me().unwrap();
        assert_eq!(me.user_id, "user1");
    }

    #[test]
    fn test_on_subscription() {
        let members = Members::new();

        let data = serde_json::json!({
            "presence": {
                "count": 2,
                "ids": ["user1", "user2"],
                "hash": {
                    "user1": {"name": "User One"},
                    "user2": {"name": "User Two"}
                }
            }
        });

        members.on_subscription(&data);

        assert_eq!(members.count(), 2);
        assert!(members.get("user1").is_some());
        assert!(members.get("user2").is_some());
    }
}
