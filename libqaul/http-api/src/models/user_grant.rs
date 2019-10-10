use japi::Attributes;
use serde_derive::{Deserialize, Serialize};

/// Returned on successful `Token` grants
///
/// The token is stored in the `id` field
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserGrant {}

impl Attributes for UserGrant {
    fn kind() -> String {
        "user_grant".into()
    }
}
