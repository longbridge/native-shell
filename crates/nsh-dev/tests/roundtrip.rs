//! Server and client talking over a real socket.

use std::{sync::mpsc, time::Duration};

use nsh_dev::{
    ClientEvent, DevClient, DevEvent, DevServer,
    protocol::{BundleFormat, LogLevel, WS_PATH},
};

const TIMEOUT: Duration = Duration::from_secs(5);

fn next_non_status(rx: &mpsc::Receiver<DevEvent>) -> DevEvent {
    loop {
        match rx.recv_timeout(TIMEOUT).expect("event") {
            DevEvent::Connected | DevEvent::Disconnected | DevEvent::Building => continue,
            event => return event,
        }
    }
}

#[test]
fn bundles_errors_and_logs_round_trip() {
    let (client_tx, client_rx) = mpsc::channel();
    let server = DevServer::start("127.0.0.1:0".parse().unwrap(), "bc-1", None, move |event| {
        let _ = client_tx.send(event);
    })
    .unwrap();
    server.publish(
        vec![1, 2, 3],
        "source v1".into(),
        Some("{\"version\":3}".into()),
        Some("{\"statusBar\":true}".into()),
        None,
    );
    let url = format!("ws://{}{WS_PATH}", server.local_addr());

    // A client with the same bytecode version gets bytecode, starting with
    // the latest bundle published before it connected.
    let (tx, rx) = mpsc::channel();
    let client = DevClient::start(url.clone(), "test-device".into(), "bc-1".into(), move |e| {
        let _ = tx.send(e);
    })
    .unwrap();
    match next_non_status(&rx) {
        DevEvent::Bundle {
            session,
            version: 1,
            format: BundleFormat::Bytecode,
            bytes,
            source_map,
            capabilities,
        } => {
            assert_ne!(session, 0, "the server names its run");
            assert_eq!(bytes, [1, 2, 3]);
            assert_eq!(source_map.as_deref(), Some("{\"version\":3}"));
            assert_eq!(capabilities.as_deref(), Some("{\"statusBar\":true}"));
        }
        other => panic!("{other:?}"),
    }
    // The server reports the host with the build it sent first.
    match client_rx.recv_timeout(TIMEOUT).unwrap() {
        ClientEvent::Connected { device, build } => {
            assert_eq!((device.as_str(), build), ("test-device", Some(1)))
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(server.clients(), 1);

    server.publish_error("syntax error at App.tsx:3");
    assert!(matches!(next_non_status(&rx), DevEvent::BuildFailed(m) if m.contains("App.tsx:3")));

    // A hot update reaches connected clients before its bundle.
    server.building();
    server.publish(
        vec![4],
        "source v2".into(),
        None,
        None,
        Some(("patch v2".into(), Some("{}".into()))),
    );
    match next_non_status(&rx) {
        DevEvent::Patch {
            version: 2,
            base: 1,
            code,
            source_map,
            ..
        } => {
            assert_eq!(code, "patch v2");
            assert_eq!(source_map.as_deref(), Some("{}"));
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        next_non_status(&rx),
        DevEvent::Bundle { version: 2, .. }
    ));

    client.log(LogLevel::Warn, "js", "hello from device");
    match client_rx.recv_timeout(TIMEOUT).unwrap() {
        ClientEvent::Log(log) => assert_eq!(
            (log.device.as_str(), log.message.as_str()),
            ("test-device", "hello from device")
        ),
        other => panic!("{other:?}"),
    }

    // A client with another engine build gets JS source instead.
    let (tx2, rx2) = mpsc::channel();
    let _other = DevClient::start(url, "old-host".into(), "bc-0".into(), move |e| {
        let _ = tx2.send(e);
    })
    .unwrap();
    match next_non_status(&rx2) {
        DevEvent::Bundle {
            format: BundleFormat::Source,
            bytes,
            ..
        } => assert_eq!(bytes, b"source v2"),
        other => panic!("{other:?}"),
    }

    // A host that goes away is reported once its session ends.
    drop(client);
    let gone = std::iter::from_fn(|| client_rx.recv_timeout(TIMEOUT).ok()).find_map(|e| match e {
        ClientEvent::Disconnected { device } => Some(device),
        _ => None,
    });
    assert_eq!(gone.as_deref(), Some("test-device"));
}

/// Build numbers start at 1 in each run of the server: a host still running
/// build 1 of an earlier run tells the new run's build 1 apart by its
/// session.
#[test]
fn each_run_of_the_server_has_its_own_session() {
    let session_of = || {
        let server =
            DevServer::start("127.0.0.1:0".parse().unwrap(), "bc-1", None, |_| {}).unwrap();
        server.publish(vec![1], "v1".into(), None, None, None);
        let url = format!("ws://{}{WS_PATH}", server.local_addr());
        let (tx, rx) = mpsc::channel();
        let _client = DevClient::start(url, "test-device".into(), "bc-1".into(), move |e| {
            let _ = tx.send(e);
        })
        .unwrap();
        match next_non_status(&rx) {
            DevEvent::Bundle {
                session,
                version: 1,
                ..
            } => session,
            other => panic!("{other:?}"),
        }
    };
    assert_ne!(session_of(), session_of());
}
