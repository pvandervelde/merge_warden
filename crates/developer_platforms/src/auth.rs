use anyhow::{anyhow, Result};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    iat: u64,
    exp: u64,
    iss: u64,
}

// pub async fn create_app_client(
//     app_id: u64,
//     private_key: &str,
//     installation_id: Option<u64>,
// ) -> Result<Octocrab> {
//     // Create JWT for GitHub App authentication
//     let now = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs();

//     let claims = JWTClaims {
//         iat: now - 60,
//         exp: now + (10 * 60), // 10 minutes expiration
//         iss: app_id,
//     };

//     let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
//         .map_err(|e| anyhow!("Invalid private key: {}", e))?;

//     let jwt = jsonwebtoken::encode(
//         &Header::new(Algorithm::RS256),
//         &claims,
//         &encoding_key,
//     )
//     .map_err(|e| anyhow!("Failed to create JWT: {}", e))?;

//     // Create an authenticated octocrab instance
//     let app_client = Octocrab::builder()
//         .personal_token(jwt)
//         .build()
//         .map_err(|e| anyhow!("Failed to build octocrab instance: {}", e))?;

//     // If installation ID is provided, get an installation token
//     if let Some(installation_id) = installation_id {
//         let installation_result = app_client
//             .apps()
//             .installation(installation_id)
//             .await;

//         let installation = installation_result.unwrap();
//         let installation_token = installation
//             .create_installation_access_token(installation_id.into())
//             .await
//             .map_err(|e| anyhow!("Failed to create installation token: {}", e))?;

//         // Create a new client with the installation token
//         let installation_client = Octocrab::builder()
//             .personal_token(installation_token.token)
//             .build()
//             .map_err(|e| anyhow!("Failed to build octocrab instance with installation token: {}", e))?;

//         Ok(installation_client)
//     } else {
//         Ok(app_client)
//     }
// }

// pub fn create_token_client(token: &str) -> Result<Octocrab> {
//     Octocrab::builder()
//         .personal_token(token.to_string())
//         .build()
//         .map_err(|e| anyhow!("Failed to build octocrab instance: {}", e))
// }
