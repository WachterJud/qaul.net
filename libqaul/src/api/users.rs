use crate::{
    error::Result,
    users::{UserProfile, UserUpdate},
    Identity, Qaul,
};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

/// A random authentication token
pub type Token = String;

/// Wrapper to encode `User` authentication state
///
/// This structure can be aquired by challenging an authentication
/// endpoint, such as `User::login` to yield a token. If a session for
/// this `Identity` already exists, it will be re-used.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserAuth(pub Identity, pub Token);

impl UserAuth {
    pub fn test() -> Self {
        Self(Identity::random(), "<fake-token>".into())
    }
}

/// Local user data and session management
///
/// Used entirely to namespace API endpoints on `Qaul` instance,
/// without having long type identifiers.
pub struct Users<'chain> {
    pub(crate) q: &'chain Qaul,
}

impl<'qaul> Users<'qaul> {
    /// Drop this scope and return back to global `Qaul` scope
    pub fn drop(&'qaul self) -> &'qaul Qaul {
        self.q
    }

    /// Enumerate all users available
    ///
    /// No information about sessions or existing login state is
    /// stored or accessible via this API.
    pub async fn list(&self) -> Vec<UserProfile> {
        self.q.users.all_local().await
    }

    /// Enumerate remote stored users available
    pub async fn list_remote(&self) -> Vec<UserProfile> {
        self.q.users.all_remote().await
    }

    /// Check if a user ID and token combination is valid
    pub async fn is_authenticated(&self, user: UserAuth) -> Result<()> {
        self.q.auth.trusted(user).map(|_| ())
    }

    /// Create a new user and authenticated session
    ///
    /// The specified password `pw` is used to encrypt the user's
    /// private key and message stores and should be kept safe from
    /// potential attackers.
    ///
    /// It's mandatory to choose a password here, however it is
    /// possible for a frontend to choose a random sequence _for_ a
    /// user, instead of leaving files completely unencrypted. In this
    /// case, there's no real security, but a drive-by will still only
    /// grab encrypted files.
    pub async fn create(&self, pw: &str) -> Result<UserAuth> {
        let keyd = self.q.sec.generate().await;
        let id = keyd.id;

        // Inform Router about new local user
        self.q.router.add_user(id).await?;
        self.q.router.online(id).await?;

        // Create user login
        self.q.users.create_local(keyd, pw).await;
        self.q.auth.set_pw(id, pw);
        let auth = self.q.auth.new_login(id, pw).map(|t| UserAuth(id, t))?;
        self.q.services.open_user(&auth).await;

        // Start announcing user profile changes
        self.q
            .announcer
            .online(&self.q.router, self.q.users.clone(), auth.0)
            .await;
        Ok(auth)
    }

    /// Delete a local user from the auth store
    ///
    /// This function requires a valid login for the user that's being
    /// deleted.  This does not delete any data associated with this
    /// user, or messages from the node (or other device nodes).
    pub async fn delete(&self, user: UserAuth) -> Result<()> {
        let id = user.0;

        // If logout succeeds, we can delete the user
        self.q.announcer.offline(id).await;
        self.logout(user).await?;
        self.q.router.del_user(id, true).await?;
        self.q.users.delete_local(id).await;
        Ok(())
    }

    /// Change the passphrase for an authenticated user
    pub fn change_pw(&self, user: UserAuth, newpw: &str) -> Result<()> {
        let (id, _) = self.q.auth.trusted(user)?;
        self.q.auth.set_pw(id, newpw);
        Ok(())
    }

    /// Create a new session login for a local User
    pub async fn login(&self, user: Identity, pw: &str) -> Result<UserAuth> {
        let token = self.q.auth.new_login(user, pw)?;
        self.q.router.online(user).await?;
        let auth = UserAuth(user, token);
        self.q.services.open_user(&auth).await;

        // This service starts syncing user profile changes across the network
        self.q
            .announcer
            .online(&self.q.router, self.q.users.clone(), auth.0)
            .await;
        Ok(auth)
    }

    /// Drop the current session Token, invalidating it
    pub async fn logout(&self, user: UserAuth) -> Result<()> {
        let (ref id, ref token) = self.q.auth.trusted(user.clone())?;
        self.q.services.close_user(&user).await;
        self.q.announcer.offline(*id).await;
        self.q.router.offline(*id).await?;
        self.q.auth.logout(id, token)?;
        Ok(())
    }

    /// Fetch the `UserProfile` for a known identity, remote or local
    ///
    /// No athentication is required for this endpoint, seeing as only
    /// public information is exposed via the `UserProfile`
    /// abstraction anyway.
    pub async fn get(&self, user: Identity) -> Result<UserProfile> {
        self.q.users.get(user).await
    }

    /// Update a `UserProfile` with a lambda, if authentication passes
    pub async fn update(&self, user: UserAuth, update: UserUpdate) -> Result<()> {
        let (id, _) = self.q.auth.trusted(user)?;
        self.q.users.modify(id, update).await
    }

    /// Validate that a `UserAuth` represents a currently logged in user
    pub fn ok(&self, user: UserAuth) -> Result<()> {
        self.q.auth.trusted(user)?;
        Ok(())
    }
}

/// A mirror of `UserAuth` used to implement the `Serialize` trait on
/// `UserAuth`
#[derive(Serialize)]
struct UserAuthSer<'a> {
    id: &'a Identity,
    token: &'a Token,
}

impl serde::ser::Serialize for UserAuth {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::ser::Serialize::serialize(
            &UserAuthSer {
                id: &self.0,
                token: &self.1,
            },
            serializer,
        )
    }
}

/// A mirror of `UserAuth` used to implement the `Deserialize` trait on
/// `UserAuth`
#[derive(Deserialize)]
struct UserAuthDe {
    id: Identity,
    token: Token,
}

impl<'de> serde::de::Deserialize<'de> for UserAuth {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ua: UserAuthDe = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(UserAuth(ua.id, ua.token))
    }
}
