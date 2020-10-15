//! libqaul user tests

mod harness;
use harness::{sec10, sec5};

use libqaul::users::UserUpdate;

#[async_std::test]
async fn user_create() {
    let net = harness::init().await;

    // There should be 0 users
    assert_eq!(net.a().users().list().await.len(), 0);

    // Create a user
    let _ = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);
}

// FIXME: currently broken
#[ignore]
#[async_std::test]
async fn user_delete() {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);

    net.a().users().delete(auth).await.unwrap();

    // There should be 0 users
    assert_eq!(net.a().users().list().await.len(), 0);
}

// FIXME: currently broken
#[ignore]
#[async_std::test]
async fn modify_user() {
    let net = harness::init().await;

    // Create a user
    let auth_a = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);

    net.a()
        .users()
        .update(
            auth_a.clone(),
            UserUpdate::DisplayName(Some("spacekookie".to_owned())),
        )
        .await
        .unwrap();

    harness::zzz(sec10()).await;
    harness::zzz(sec10()).await;
    harness::zzz(sec10()).await;
    harness::zzz(sec10()).await;

    let profile = net.b().users().get(auth_a.0).await.unwrap();
    assert_eq!(profile.display_name, Some("spacekookie".to_owned()));
}

#[async_std::test]
async fn fail_delete() {
    use libqaul::{users::UserAuth, Identity};

    let net = harness::init().await;

    // Delete a user but it fails
    assert!(net
        .a()
        .users()
        .delete(UserAuth(Identity::random(), "<fake-taken>".into()))
        .await
        .is_err());
}

#[async_std::test]
async fn change_pw() {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);

    net.a()
        .users()
        .change_pw(auth, "new and better password")
        .unwrap();
}

#[async_std::test]
async fn logout_login() {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await.unwrap();
    let id = auth.0;
    assert_eq!(net.a().users().list().await.len(), 1);

    // Yield user session
    net.a().users().logout(auth).await.unwrap();

    // Login again
    net.a().users().login(id, "abcdefg").await.unwrap();
}

#[async_std::test]
async fn login_changed_pw() {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await.unwrap();
    let id = auth.0;
    assert_eq!(net.a().users().list().await.len(), 1);

    net.a()
        .users()
        .change_pw(auth.clone(), "new and better password")
        .unwrap();

    // Yield user session
    net.a().users().logout(auth).await.unwrap();

    // Login again
    net.a()
        .users()
        .login(id, "new and better password")
        .await
        .unwrap();
}

#[async_std::test]
async fn get_user_profile() {
    use libqaul::users::UserProfile;
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);

    let profile = net.a().users().get(auth.0).await.unwrap();
    assert_eq!(
        profile,
        UserProfile {
            id: auth.0,
            display_name: None,
            real_name: None,
            bio: Default::default(),
            services: Default::default(),
            avatar: None,
        }
    );
}

#[async_std::test]
async fn simple_network_announce() {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let net = harness::init().await;

    // Create a user on node A
    let auth = net.a().users().create("abcdefg").await.unwrap();
    assert_eq!(net.a().users().list().await.len(), 1);

    let t1 = Instant::now();
    harness::timeout(sec5(), async {
        let b = Arc::clone(net.b());
        loop {
            harness::zzz(Duration::from_millis(20)).await;
            if b.users().list_remote().await.len() != 0 {
                let diff = Instant::now() - t1;
                println!("Listened for {} millis", diff.as_millis());
                break;
            }
        }
    })
    .await
    .unwrap();
}

#[async_std::test]
async fn simple_network_announce_reverse() {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let net = harness::init().await;

    // Create a user on node A
    let _auth = net.a().users().create("abcdefg").await.unwrap();

    // And then on b
    let _auth = net.b().users().create("abcdefg").await.unwrap();
    assert_eq!(net.b().users().list().await.len(), 1);

    let t1 = Instant::now();
    harness::timeout(sec5(), async {
        let a = Arc::clone(net.a());
        loop {
            harness::zzz(Duration::from_millis(20)).await;
            if a.users().list_remote().await.len() != 0 {
                let diff = Instant::now() - t1;
                println!("Listened for {} millis", diff.as_millis());
                break;
            }
        }
    })
    .await
    .unwrap();
}
