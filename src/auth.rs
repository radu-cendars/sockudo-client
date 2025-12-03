//! HTTP-based authorization for private and presence channels.
//!
//! This module implements the Pusher authorization protocol for private and presence channels,
//! as well as user authentication.

use crate::channels::ChannelAuthData;
use crate::error::{Result, SockudoError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request body for channel authorization
#[derive(Debug, Serialize)]
struct AuthRequest {
    socket_id: String,
    channel_name: String,
}

/// Response from authorization endpoint
#[derive(Debug, Deserialize)]
struct AuthResponse {
    auth: String,
    #[serde(default)]
    channel_data: Option<String>,
    #[serde(default)]
    shared_secret: Option<String>,
}

/// Request body for user authentication
#[derive(Debug, Serialize)]
struct UserAuthRequest {
    socket_id: String,
}

/// Response from user authentication endpoint
#[derive(Debug, Deserialize)]
struct UserAuthResponse {
    auth: String,
    user_data: String,
}

/// User authentication data
#[derive(Debug, Clone)]
pub struct UserAuthData {
    pub auth: String,
    pub user_data: String,
}

/// HTTP client for authorization requests
pub struct AuthClient {
    auth_endpoint: Option<String>,
    auth_headers: HashMap<String, String>,
    user_auth_endpoint: Option<String>,
    user_auth_headers: HashMap<String, String>,
}

impl AuthClient {
    /// Create a new authorization client
    pub fn new(
        auth_endpoint: Option<String>,
        auth_headers: Option<HashMap<String, String>>,
        user_auth_endpoint: Option<String>,
        user_auth_headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            auth_endpoint,
            auth_headers: auth_headers.unwrap_or_default(),
            user_auth_endpoint,
            user_auth_headers: user_auth_headers.unwrap_or_default(),
        }
    }

    /// Authorize a channel subscription (async)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn authorize_channel(
        &self,
        channel_name: &str,
        socket_id: &str,
    ) -> Result<ChannelAuthData> {
        let endpoint = self.auth_endpoint.as_ref().ok_or_else(|| {
            SockudoError::authorization("No auth_endpoint configured for private/presence channels")
        })?;

        // Build request body as form data
        let params = [("socket_id", socket_id), ("channel_name", channel_name)];

        // Make async HTTP POST request
        let client = reqwest::Client::new();
        let mut request = client.post(endpoint).form(&params);

        // Add custom headers
        for (key, value) in &self.auth_headers {
            request = request.header(key, value);
        }

        // Send request and parse response
        let response = request.send().await.map_err(|e| {
            SockudoError::authorization(format!("Failed to send authorization request: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(SockudoError::authorization(format!(
                "Authorization failed with status: {}",
                response.status()
            )));
        }

        let auth_response: AuthResponse = response.json().await.map_err(|e| {
            SockudoError::authorization(format!("Failed to parse authorization response: {}", e))
        })?;

        Ok(ChannelAuthData {
            auth: auth_response.auth,
            channel_data: auth_response.channel_data,
            shared_secret: auth_response.shared_secret,
        })
    }

    /// Authorize a channel subscription (WASM version)
    #[cfg(target_arch = "wasm32")]
    pub async fn authorize_channel(
        &self,
        channel_name: &str,
        socket_id: &str,
    ) -> Result<ChannelAuthData> {
        let endpoint = self.auth_endpoint.as_ref().ok_or_else(|| {
            SockudoError::authorization("No auth_endpoint configured for private/presence channels")
        })?;

        // Build request body as form data
        let form_data = web_sys::FormData::new()
            .map_err(|_| SockudoError::authorization("Failed to create form data"))?;

        form_data
            .append_with_str("socket_id", socket_id)
            .map_err(|_| SockudoError::authorization("Failed to append socket_id"))?;

        form_data
            .append_with_str("channel_name", channel_name)
            .map_err(|_| SockudoError::authorization("Failed to append channel_name"))?;

        // Make HTTP POST request using gloo-net
        let mut request = gloo_net::http::Request::post(endpoint);

        // Add custom headers
        for (key, value) in &self.auth_headers {
            request = request.header(key, value);
        }

        let request = request
            .body(form_data)
            .map_err(|e| SockudoError::authorization(format!("Failed to build request: {}", e)))?;

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| SockudoError::authorization(format!("Failed to send request: {}", e)))?;

        if !response.ok() {
            return Err(SockudoError::authorization(format!(
                "Authorization failed with status: {}",
                response.status()
            )));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| SockudoError::authorization(format!("Failed to parse response: {}", e)))?;

        Ok(ChannelAuthData {
            auth: auth_response.auth,
            channel_data: auth_response.channel_data,
            shared_secret: auth_response.shared_secret,
        })
    }

    /// Authenticate a user (async)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn authenticate_user(&self, socket_id: &str) -> Result<UserAuthData> {
        let endpoint = self
            .user_auth_endpoint
            .as_ref()
            .ok_or_else(|| SockudoError::authorization("No user_auth_endpoint configured"))?;

        // Build request body as form data
        let params = [("socket_id", socket_id)];

        // Make async HTTP POST request
        let client = reqwest::Client::new();
        let mut request = client.post(endpoint).form(&params);

        // Add custom headers
        for (key, value) in &self.user_auth_headers {
            request = request.header(key, value);
        }

        // Send request and parse response
        let response = request.send().await.map_err(|e| {
            SockudoError::authorization(format!("Failed to send user auth request: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(SockudoError::authorization(format!(
                "User authentication failed with status: {}",
                response.status()
            )));
        }

        let auth_response: UserAuthResponse = response.json().await.map_err(|e| {
            SockudoError::authorization(format!("Failed to parse user auth response: {}", e))
        })?;

        Ok(UserAuthData {
            auth: auth_response.auth,
            user_data: auth_response.user_data,
        })
    }

    /// Authenticate a user (WASM version)
    #[cfg(target_arch = "wasm32")]
    pub async fn authenticate_user(&self, socket_id: &str) -> Result<UserAuthData> {
        let endpoint = self
            .user_auth_endpoint
            .as_ref()
            .ok_or_else(|| SockudoError::authorization("No user_auth_endpoint configured"))?;

        // Build request body as form data
        let form_data = web_sys::FormData::new()
            .map_err(|_| SockudoError::authorization("Failed to create form data"))?;

        form_data
            .append_with_str("socket_id", socket_id)
            .map_err(|_| SockudoError::authorization("Failed to append socket_id"))?;

        // Make HTTP POST request
        let mut request = gloo_net::http::Request::post(endpoint);

        // Add custom headers
        for (key, value) in &self.user_auth_headers {
            request = request.header(key, value);
        }

        let request = request
            .body(form_data)
            .map_err(|e| SockudoError::authorization(format!("Failed to build request: {}", e)))?;

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| SockudoError::authorization(format!("Failed to send request: {}", e)))?;

        if !response.ok() {
            return Err(SockudoError::authorization(format!(
                "User authentication failed with status: {}",
                response.status()
            )));
        }

        let auth_response: UserAuthResponse = response
            .json()
            .await
            .map_err(|e| SockudoError::authorization(format!("Failed to parse response: {}", e)))?;

        Ok(UserAuthData {
            auth: auth_response.auth,
            user_data: auth_response.user_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_client_creation() {
        let client = AuthClient::new(
            Some("http://localhost:3000/pusher/auth".to_string()),
            None,
            Some("http://localhost:3000/pusher/user-auth".to_string()),
            None,
        );

        assert!(client.auth_endpoint.is_some());
        assert!(client.user_auth_endpoint.is_some());
    }
}
