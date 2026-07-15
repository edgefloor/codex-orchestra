//! PROTOTYPE ONLY: deterministic issue #20 host/replay fixture.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Write};

pub const CODEX_REVISION: &str = "f90e7deea6a715bbd153044af6f475eefa749177";
pub const T3CODE_REVISION: &str = "ecb35f75839925dd1ac6f854efeef5c9e291d11b";
pub const RENDERER_BUILD: &str = "t3code-ecb35f75-orchestra-prototype";
pub const HOST_BUILD: &str = "orchestra-host-issue-20-prototype";
pub const PROTOCOL_SCHEMA: &str = "codex-app-server+orchestra-prototype-v1";
pub const SNAPSHOT_SCHEMA: &str = "task-snapshot-prototype-v1";
pub const MAX_FRAME_BYTES: usize = 64 * 1024;
pub const DELIVERY_QUEUE_LIMIT: usize = 4;
pub const REPLAY_TAIL_LIMIT: usize = 16;
pub const MAX_FLOOD_EVENTS: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame length {0} exceeds {MAX_FRAME_BYTES} bytes")]
    TooLarge(usize),
    #[error("truncated frame")]
    Truncated,
    #[error("invalid UTF-8 JSON frame: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("frame I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_frame(reader: &mut impl Read) -> Result<Option<Value>, FrameError> {
    let mut length = [0_u8; 4];
    if reader.read(&mut length[..1])? == 0 {
        return Ok(None);
    }
    reader
        .read_exact(&mut length[1..])
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::UnexpectedEof => FrameError::Truncated,
            _ => FrameError::Io(error),
        })?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge(length));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|error| {
        if error.kind() == std::io::ErrorKind::UnexpectedEof {
            FrameError::Truncated
        } else {
            FrameError::Io(error)
        }
    })?;
    Ok(Some(serde_json::from_slice(&body)?))
}

pub fn write_frame(writer: &mut impl Write, value: &Value) -> Result<(), FrameError> {
    let body = serde_json::to_vec(value)?;
    if body.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge(body.len()));
    }
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct HostAction {
    pub data: Vec<Value>,
    pub control_challenge: Option<Value>,
    pub close: bool,
}

impl HostAction {
    fn data(data: Vec<Value>) -> Self {
        Self {
            data,
            control_challenge: None,
            close: false,
        }
    }
}

#[derive(Debug, Clone)]
struct TaskFixture {
    thread: Value,
    projection: Value,
    journal: Vec<Value>,
    next_sequence: u64,
}

impl TaskFixture {
    fn append_event(&mut self, event: Value) {
        self.journal.push(event);
        if self.journal.len() > REPLAY_TAIL_LIMIT {
            self.journal.remove(0);
        }
    }
}

#[derive(Debug)]
pub struct PrototypeHost {
    host_instance_id: String,
    initialized: bool,
    tasks: BTreeMap<String, TaskFixture>,
    subscriptions: BTreeMap<String, u64>,
}

impl PrototypeHost {
    pub fn new(host_instance_id: impl Into<String>) -> Self {
        let parent_item = orchestra_item(1, "running");
        let parent_thread = json!({
            "id": "task-parent",
            "title": "Issue #20 parent task",
            "parentThreadId": null,
            "turns": [{
                "id": "turn-parent",
                "status": "inProgress",
                "items": [
                    {"type": "userMessage", "id": "user-1", "content": [{"type": "text", "text": "Run the fixture workflow"}]},
                    parent_item
                ]
            }]
        });
        let child_thread = json!({
            "id": "task-child",
            "title": "Canonical child task",
            "parentThreadId": "task-parent",
            "turns": [{
                "id": "turn-child",
                "status": "completed",
                "items": [
                    {"type": "agentMessage", "id": "child-message-1", "text": "child detail stays here"},
                    {"type": "commandExecution", "id": "child-command-1", "command": "cargo test", "status": "completed"}
                ]
            }]
        });
        let digest = digest(1, "run active", 0);
        let parent_projection = json!({
            "items": [parent_item],
            "worldState": {"orchestra.runDigest": digest},
            "queryRevision": 1
        });
        let child_projection = json!({"items": [], "worldState": {}, "queryRevision": 1});
        let parent_journal = vec![notification(
            "orchestra/lifecycle/updated",
            "task-parent",
            1,
            json!({"item": orchestra_item(1, "running")}),
        )];
        let child_journal = vec![notification(
            "item/completed",
            "task-child",
            1,
            json!({"item": {"type": "agentMessage", "id": "child-message-1", "text": "child detail stays here"}}),
        )];
        let tasks = BTreeMap::from([
            (
                "task-parent".into(),
                TaskFixture {
                    thread: parent_thread,
                    projection: parent_projection,
                    journal: parent_journal,
                    next_sequence: 2,
                },
            ),
            (
                "task-child".into(),
                TaskFixture {
                    thread: child_thread,
                    projection: child_projection,
                    journal: child_journal,
                    next_sequence: 2,
                },
            ),
        ]);
        Self {
            host_instance_id: host_instance_id.into(),
            initialized: false,
            tasks,
            subscriptions: BTreeMap::new(),
        }
    }

    pub fn handle(&mut self, request: Value) -> HostAction {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

        if !self.initialized && method != "initialize" {
            return HostAction::data(vec![error(
                id,
                -32002,
                "not_initialized",
                "Initialize must be the first request",
                "never",
                json!({}),
            )]);
        }

        match method {
            "initialize" => self.initialize(id, params),
            "thread/list" => HostAction::data(vec![response(
                id,
                json!({
                    "data": self.tasks.values().map(|task| json!({"id": task.thread["id"], "title": task.thread["title"]})).collect::<Vec<_>>(),
                    "nextCursor": null
                }),
            )]),
            "thread/read" => self.thread_read(id, params),
            "host/snapshot/get" => self.snapshot(id, params),
            "host/replay/subscribe" => self.subscribe(id, params),
            "orchestra/query" => self.query(id, params),
            "host/diagnostics/export" => HostAction::data(vec![response(
                id,
                json!({
                    "events": [{"kind": "fixture", "token": "[REDACTED]", "command": "cargo test"}],
                    "redacted": true
                }),
            )]),
            "prototype/advance" => self.advance(id),
            "prototype/flood" => self.flood(id, params),
            "prototype/confirm" => HostAction {
                data: Vec::new(),
                control_challenge: Some(json!({
                    "type": "confirmationChallenge",
                    "challengeId": "challenge-1",
                    "requestDigest": sha256_hex(b"protected fixture effect"),
                    "consequence": "Allow the protected fixture effect?",
                    "stateRevision": 2,
                    "expiresInMs": 5000
                })),
                close: false,
            },
            _ => HostAction::data(vec![error(
                id,
                -32601,
                "method_not_found",
                "Method not found",
                "never",
                json!({"method": method}),
            )]),
        }
    }

    pub fn confirmation_response(&self, request_id: Value, control: Value) -> Value {
        let accepted = control.get("challengeId") == Some(&json!("challenge-1"))
            && control.get("decision") == Some(&json!("accept"));
        if accepted {
            response(
                request_id,
                json!({"confirmed": true, "channel": "privilegedControlPipe"}),
            )
        } else {
            error(
                request_id,
                -32020,
                "confirmation_denied",
                "Protected confirmation was denied",
                "never",
                json!({}),
            )
        }
    }

    fn initialize(&mut self, id: Value, params: Value) -> HostAction {
        let expected = compatibility_tuple();
        let supplied = params.get("compatibility").cloned().unwrap_or(Value::Null);
        if supplied != expected {
            return HostAction {
                data: vec![error(
                    id,
                    -32001,
                    "incompatible_bundle",
                    "Renderer and host compatibility tuple differ",
                    "never",
                    json!({"expected": expected, "received": supplied}),
                )],
                control_challenge: None,
                close: true,
            };
        }
        self.initialized = true;
        self.subscriptions.clear();
        HostAction::data(vec![response(
            id,
            json!({
                "compatibility": compatibility_tuple(),
                "hostInstanceId": self.host_instance_id,
                "capabilities": ["thread/read", "host/snapshot", "host/replay", "orchestra/threadItem", "orchestra/worldState", "orchestra/query", "host/privilegedConfirmation"],
                "limits": {"frameBytes": MAX_FRAME_BYTES, "deliveryQueue": DELIVERY_QUEUE_LIMIT, "replayBatch": 32}
            }),
        )])
    }

    fn thread_read(&self, id: Value, params: Value) -> HostAction {
        let task_id = params
            .get("threadId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match self.tasks.get(task_id) {
            Some(task) => HostAction::data(vec![response(id, json!({"thread": task.thread}))]),
            None => HostAction::data(vec![error(
                id,
                -32004,
                "task_not_found",
                "Task not found",
                "never",
                json!({"taskId": task_id}),
            )]),
        }
    }

    fn snapshot(&self, id: Value, params: Value) -> HostAction {
        let task_id = params
            .get("taskId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match self.tasks.get(task_id) {
            Some(task) => {
                let barrier = task
                    .journal
                    .last()
                    .and_then(|event| event.pointer("/orchestra/sequence"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let snapshot = json!({
                    "schema": SNAPSHOT_SCHEMA,
                    "taskId": task_id,
                    "threadRead": {"thread": task.thread},
                    "orchestra": task.projection,
                    "barrier": barrier
                });
                let digest =
                    sha256_hex(&serde_json::to_vec(&snapshot).expect("snapshot serializes"));
                HostAction::data(vec![response(
                    id,
                    json!({"snapshot": snapshot, "digest": digest}),
                )])
            }
            None => HostAction::data(vec![error(
                id,
                -32004,
                "task_not_found",
                "Task not found",
                "never",
                json!({"taskId": task_id}),
            )]),
        }
    }

    fn subscribe(&mut self, id: Value, params: Value) -> HostAction {
        let task_id = params
            .get("taskId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(task) = self.tasks.get(task_id) else {
            return HostAction::data(vec![error(
                id,
                -32004,
                "task_not_found",
                "Task not found",
                "never",
                json!({"taskId": task_id}),
            )]);
        };
        let cursor = params.get("cursor").and_then(Value::as_u64).unwrap_or(0);
        let oldest = task
            .journal
            .first()
            .and_then(|event| event.pointer("/orchestra/sequence"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let newest = task
            .journal
            .last()
            .and_then(|event| event.pointer("/orchestra/sequence"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if cursor.saturating_add(1) < oldest || cursor > newest {
            return HostAction::data(vec![error(
                id,
                -32010,
                "snapshot_required",
                "Replay cursor is not retained by this host",
                "after_snapshot",
                json!({"oldestRetainedCursor": oldest, "newestCursor": newest}),
            )]);
        }
        let mut data = vec![response(
            id,
            json!({"taskId": task_id, "cursor": cursor, "liveAfter": newest}),
        )];
        data.extend(
            task.journal
                .iter()
                .filter(|event| {
                    event
                        .pointer("/orchestra/sequence")
                        .and_then(Value::as_u64)
                        .is_some_and(|sequence| sequence > cursor)
                })
                .cloned(),
        );
        self.subscriptions.insert(task_id.to_string(), newest);
        HostAction::data(data)
    }

    fn query(&self, id: Value, params: Value) -> HostAction {
        if params.get("taskId") != Some(&json!("task-parent")) {
            return HostAction::data(vec![error(
                id,
                -32003,
                "authorization_denied",
                "The query is outside the authorized parent task",
                "never",
                json!({}),
            )]);
        }
        let selector = params
            .get("selector")
            .cloned()
            .unwrap_or_else(|| json!({"kind": "run", "id": "run-1"}));
        let consumer = params
            .get("consumer")
            .and_then(Value::as_str)
            .unwrap_or("renderer");
        HostAction::data(vec![response(
            id,
            json!({
                "consumer": consumer,
                "selection": {"selector": selector, "revision": 2, "items": [{"id": "step-1", "status": "completed", "childTaskId": "task-child"}]},
                "budget": {"maxItems": 8, "truncated": false},
                "authorized": true
            }),
        )])
    }

    fn advance(&mut self, id: Value) -> HostAction {
        let task = self.tasks.get_mut("task-parent").expect("fixture parent");
        let sequence = task.next_sequence;
        task.next_sequence += 1;
        let item = orchestra_item(2, "completed");
        let digest = digest(2, "run completed", 1);
        task.projection["items"] = json!([item]);
        task.projection["worldState"]["orchestra.runDigest"] = digest.clone();
        task.projection["queryRevision"] = json!(2);
        let event = notification(
            "orchestra/lifecycle/updated",
            "task-parent",
            sequence,
            json!({
                "item": item,
                "worldState": {"key": "orchestra.runDigest", "operation": "replace", "value": digest}
            }),
        );
        task.append_event(event.clone());
        let mut data = vec![response(id, json!({"advancedTo": sequence}))];
        if self.subscriptions.contains_key("task-parent") {
            data.push(event);
            self.subscriptions.insert("task-parent".into(), sequence);
        }
        HostAction::data(data)
    }

    fn flood(&mut self, id: Value, params: Value) -> HostAction {
        let count = params.get("count").and_then(Value::as_u64).unwrap_or(0) as usize;
        if count > MAX_FLOOD_EVENTS {
            return HostAction::data(vec![error(
                id,
                -32011,
                "limit_exceeded",
                "Requested fixture event count exceeds the hard limit",
                "never",
                json!({"limit": MAX_FLOOD_EVENTS}),
            )]);
        }
        let task = self.tasks.get_mut("task-parent").expect("fixture parent");
        for index in 0..count {
            let sequence = task.next_sequence;
            task.next_sequence += 1;
            task.append_event(notification(
                "prototype/noisy",
                "task-parent",
                sequence,
                json!({"index": index}),
            ));
        }
        if count > DELIVERY_QUEUE_LIMIT {
            let last_journaled_sequence = task.next_sequence - 1;
            let reconnect = json!({
                "jsonrpc": "2.0",
                "method": "host/reconnectRequired",
                "params": {
                    "code": "slow_consumer",
                    "retry": "after_reconnect",
                    "lastJournaledSequence": last_journaled_sequence
                }
            });
            HostAction::data(vec![
                response(
                    id,
                    json!({"journaled": count, "delivered": 0, "sessionReset": true}),
                ),
                reconnect,
            ])
        } else {
            let start = task.journal.len().saturating_sub(count);
            let mut data = vec![response(
                id,
                json!({"journaled": count, "delivered": count, "sessionReset": false}),
            )];
            data.extend(task.journal[start..].iter().cloned());
            HostAction::data(data)
        }
    }
}

pub fn compatibility_tuple() -> Value {
    json!({
        "rendererBuildId": RENDERER_BUILD,
        "hostBuildId": HOST_BUILD,
        "codexRevision": CODEX_REVISION,
        "protocolSchemaHash": sha256_hex(PROTOCOL_SCHEMA.as_bytes()),
        "snapshotSchemaId": SNAPSHOT_SCHEMA,
        "requiredCapabilities": ["thread/read", "host/snapshot", "host/replay", "orchestra/threadItem", "orchestra/worldState", "orchestra/query", "host/privilegedConfirmation"]
    })
}

fn orchestra_item(revision: u64, status: &str) -> Value {
    json!({
        "type": "orchestra",
        "item": {
            "kind": "step",
            "id": "orchestra-step-1",
            "revision": revision,
            "runId": "run-1",
            "stepId": "step-1",
            "status": status,
            "summary": if status == "completed" { "fixture child completed" } else { "fixture child active" },
            "childTaskId": "task-child"
        }
    })
}

fn digest(revision: u64, summary: &str, outcomes: u64) -> Value {
    json!({"revision": revision, "summary": summary, "requiredActions": [], "active": [], "outcomes": outcomes, "omitted": 0})
}

fn notification(method: &str, task_id: &str, sequence: u64, params: Value) -> Value {
    json!({"jsonrpc": "2.0", "method": method, "params": params, "orchestra": {"taskId": task_id, "sequence": sequence}})
}

fn response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error(
    id: Value,
    code: i64,
    safe_code: &str,
    message: &str,
    retry: &str,
    details: Value,
) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message, "data": {"code": safe_code, "retry": retry, "details": details}}})
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn request(id: i64, method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
    }

    fn initialized_host() -> PrototypeHost {
        let mut host = PrototypeHost::new("test-host");
        let action = host.handle(request(
            1,
            "initialize",
            json!({"compatibility": compatibility_tuple()}),
        ));
        assert_eq!(action.data[0]["result"]["hostInstanceId"], "test-host");
        host
    }

    #[test]
    fn bounded_frame_round_trip_and_rejection() {
        let value = request(1, "thread/read", json!({"threadId": "task-parent"}));
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &value).unwrap();
        assert_eq!(read_frame(&mut Cursor::new(bytes)).unwrap(), Some(value));

        let mut oversized = Cursor::new(((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(matches!(
            read_frame(&mut oversized),
            Err(FrameError::TooLarge(_))
        ));
    }

    #[test]
    fn exact_bundle_mismatch_is_terminal_and_initialize_has_no_task_cursors() {
        let mut host = PrototypeHost::new("test-host");
        let mismatch = host.handle(request(
            1,
            "initialize",
            json!({"compatibility": {"rendererBuildId": "wrong"}}),
        ));
        assert!(mismatch.close);
        assert_eq!(
            mismatch.data[0]["error"]["data"]["code"],
            "incompatible_bundle"
        );

        let mut host = PrototypeHost::new("test-host");
        let accepted = host.handle(request(
            1,
            "initialize",
            json!({"compatibility": compatibility_tuple()}),
        ));
        assert!(accepted.data[0]["result"].get("taskCursors").is_none());
    }

    #[test]
    fn snapshot_barrier_replay_and_expired_cursor_are_task_local() {
        let mut host = initialized_host();
        let snapshot = host.handle(request(
            2,
            "host/snapshot/get",
            json!({"taskId": "task-parent"}),
        ));
        assert_eq!(snapshot.data[0]["result"]["snapshot"]["barrier"], 1);
        host.handle(request(3, "prototype/advance", json!({})));
        let replay = host.handle(request(
            4,
            "host/replay/subscribe",
            json!({"taskId": "task-parent", "cursor": 1}),
        ));
        assert_eq!(replay.data[1]["orchestra"]["sequence"], 2);
        let expired = host.handle(request(
            5,
            "host/replay/subscribe",
            json!({"taskId": "task-parent", "cursor": 99}),
        ));
        assert_eq!(
            expired.data[0]["error"]["data"]["code"],
            "snapshot_required"
        );
    }

    #[test]
    fn parent_projection_references_child_without_copying_child_detail() {
        let host = initialized_host();
        let parent = host.thread_read(json!(1), json!({"threadId": "task-parent"}));
        let encoded = serde_json::to_string(&parent.data).unwrap();
        assert!(encoded.contains("task-child"));
        assert!(!encoded.contains("child detail stays here"));
        let child = host.thread_read(json!(2), json!({"threadId": "task-child"}));
        assert!(
            serde_json::to_string(&child.data)
                .unwrap()
                .contains("child detail stays here")
        );
    }

    #[test]
    fn query_adapters_share_selection_and_overload_journals_before_reset() {
        let mut host = initialized_host();
        let renderer = host.query(
            json!(1),
            json!({"taskId": "task-parent", "consumer": "renderer", "selector": {"kind": "run", "id": "run-1"}}),
        );
        let model = host.query(
            json!(2),
            json!({"taskId": "task-parent", "consumer": "nativeTaskTool", "selector": {"kind": "run", "id": "run-1"}}),
        );
        assert_eq!(
            renderer.data[0]["result"]["selection"],
            model.data[0]["result"]["selection"]
        );
        let flood = host.flood(json!(3), json!({"count": DELIVERY_QUEUE_LIMIT + 1}));
        assert_eq!(
            flood.data[0]["result"]["journaled"],
            (DELIVERY_QUEUE_LIMIT + 1) as u64
        );
        assert_eq!(flood.data[1]["params"]["code"], "slow_consumer");
        let limited = host.flood(json!(31), json!({"count": MAX_FLOOD_EVENTS + 1}));
        assert_eq!(limited.data[0]["error"]["data"]["code"], "limit_exceeded");
        assert!(host.tasks["task-parent"].journal.len() <= REPLAY_TAIL_LIMIT);

        let denied = host.query(
            json!(4),
            json!({"taskId": "task-child", "consumer": "renderer", "selector": {"kind": "run", "id": "run-1"}}),
        );
        assert_eq!(
            denied.data[0]["error"]["data"]["code"],
            "authorization_denied"
        );
        let diagnostics = host.handle(request(5, "host/diagnostics/export", json!({})));
        let encoded = serde_json::to_string(&diagnostics.data).unwrap();
        assert!(encoded.contains("[REDACTED]"));
        assert!(!encoded.contains("secret-token"));
    }
}
