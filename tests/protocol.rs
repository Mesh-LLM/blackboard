#![cfg(unix)]
use mesh_llm_plugin::{LocalStream, proto, read_envelope, write_envelope};
use std::time::Duration;
use tokio::{net::UnixListener, process::Command};

#[tokio::test]
async fn protocol_three_initialization_health_and_disconnect() {
    let sockets = tempfile::tempdir_in("/tmp").unwrap();
    let socket = sockets.path().join("ipc");
    let listener = UnixListener::bind(&socket).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_blackboard"));
    let mut child = command
        .env("MESH_LLM_PLUGIN_ENDPOINT", &socket)
        .env("MESH_LLM_PLUGIN_TRANSPORT", "unix")
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let (stream, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
        .await
        .unwrap()
        .unwrap();
    let mut stream = LocalStream::Unix(stream);
    // Deliberately literal: a future SDK bump must not silently change this contract.
    write_envelope(
        &mut stream,
        &proto::Envelope {
            protocol_version: 3,
            plugin_id: "blackboard".into(),
            request_id: 1,
            payload: Some(proto::envelope::Payload::InitializeRequest(
                proto::InitializeRequest {
                    host_protocol_version: 3,
                    host_version: "0.77.0".into(),
                    host_info_json: "{}".into(),
                    mesh_visibility: proto::MeshVisibility::Private as i32,
                },
            )),
        },
    )
    .await
    .unwrap();
    let reply = tokio::time::timeout(Duration::from_secs(5), read_envelope(&mut stream))
        .await
        .unwrap()
        .unwrap();
    let Some(proto::envelope::Payload::InitializeResponse(initialized)) = reply.payload else {
        panic!("expected initialization response");
    };
    assert_eq!(initialized.plugin_protocol_version, 3);
    assert_eq!(initialized.plugin_id, "blackboard");
    assert!(initialized.manifest.is_some());
    write_envelope(
        &mut stream,
        &proto::Envelope {
            protocol_version: 3,
            plugin_id: "blackboard".into(),
            request_id: 2,
            payload: Some(proto::envelope::Payload::HealthRequest(
                proto::HealthRequest {},
            )),
        },
    )
    .await
    .unwrap();
    // Plugins may emit notifications before replying to health.
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let reply = read_envelope(&mut stream).await.unwrap();
            if let Some(proto::envelope::Payload::HealthResponse(health)) = reply.payload {
                assert_eq!(reply.request_id, 2);
                assert_eq!(health.status, proto::health_response::Status::Ok as i32);
                break;
            }
        }
    })
    .await
    .unwrap();
    drop(stream);
    tokio::time::timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap();
}
