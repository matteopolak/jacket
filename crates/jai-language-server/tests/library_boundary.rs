//! JSON-RPC envelope checks precede the standard library's untagged decoding.
use jai_language_server::{JsonSession, Limits};
use serde_json::Value;

fn rejected(input: &str, expected_id: Value) {
    let mut session = JsonSession::new(Limits::default());
    let answers = session.handle_json(input).unwrap();
    assert_eq!(answers.len(), 1);
    let answer: Value = serde_json::from_str(&answers[0]).unwrap();
    assert_eq!(answer["error"]["code"], -32600, "{input}: {answer}");
    assert_eq!(answer["id"], expected_id);
    let next = session
        .handle_json(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        )
        .unwrap();
    let next: Value = serde_json::from_str(&next[0]).unwrap();
    assert!(
        next.get("result").is_some(),
        "invalid input changed lifecycle: {next}"
    );
}

#[test]
fn invalid_standard_ids_cannot_become_notifications() {
    for id in ["null", "2147483648", "1.0", "true", "[]", "{}"] {
        rejected(
            &format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"initialize","params":{{"capabilities":{{}}}}}}"#
            ),
            Value::Null,
        );
    }
}

#[test]
fn contradictory_and_duplicate_reserved_members_are_rejected() {
    for input in [
        r#"{"jsonrpc":"2.0","jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","id":1,"id":2,"method":"initialize","params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","result":null,"params":{"capabilities":{}}}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":null,"error":{"code":-1,"message":"bad"}}"#,
    ] {
        let expected_id = if input.contains(r#""id":2"#) {
            Value::Null
        } else {
            Value::from(1)
        };
        rejected(input, expected_id);
    }
}

#[test]
fn initialized_notification_is_validated_before_running() {
    let mut session = JsonSession::new(Limits::default());
    session
        .handle_json(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        )
        .unwrap();
    let early = session
        .handle_json(r#"{"jsonrpc":"2.0","id":2,"method":"unknown"}"#)
        .unwrap();
    let early: Value = serde_json::from_str(&early[0]).unwrap();
    assert_eq!(early["error"]["code"], -32002);
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":[]}"#)
        .unwrap();
    let early = session
        .handle_json(r#"{"jsonrpc":"2.0","id":3,"method":"unknown"}"#)
        .unwrap();
    let early: Value = serde_json::from_str(&early[0]).unwrap();
    assert_eq!(early["error"]["code"], -32002);
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
        .unwrap();
    let ready = session
        .handle_json(r#"{"jsonrpc":"2.0","id":4,"method":"unknown"}"#)
        .unwrap();
    let ready: Value = serde_json::from_str(&ready[0]).unwrap();
    assert_eq!(ready["error"]["code"], -32601);
}

#[test]
fn parse_error_with_an_early_id_still_has_null_id() {
    let mut session = JsonSession::new(Limits::default());
    for text in [
        r#"{"jsonrpc":"2.0","id":7,"method":"exit",}"#,
        r#"{"jsonrpc":"2.0","id":7,"method":"exit"} {}"#,
    ] {
        let out = session.handle_json(text).unwrap();
        let out: Value = serde_json::from_str(&out[0]).unwrap();
        assert_eq!(out["id"], Value::Null);
        assert_eq!(out["error"]["code"], -32700);
        assert_eq!(session.exit_status(), None);
    }
}

#[test]
fn batch_cancellation_only_matches_previously_arrived_pending_ids() {
    let mut session = JsonSession::new(Limits::default());
    session
        .handle_json(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        )
        .unwrap();
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
        .unwrap();
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#.to_string();
    let cancel = r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}"#.to_string();
    for (batch, response_at, expected) in [
        (vec![cancel.clone(), request.clone()], 1, -32601),
        (vec![request.clone(), cancel.clone()], 0, -32800),
        (vec![cancel, request], 1, -32601),
    ] {
        let output = session.handle_batch(&batch).unwrap();
        let response: Value = serde_json::from_str(&output[response_at][0]).unwrap();
        assert_eq!(response["error"]["code"], expected);
    }
}

#[test]
fn failed_or_exited_batch_retires_its_pending_registrations() {
    let mut session = JsonSession::new(Limits {
        output_bytes: 128,
        ..Limits::default()
    });
    let oversized_id = "x".repeat(512);
    let batch = [
        serde_json::json!({"jsonrpc":"2.0","id":oversized_id,"method":"unknown"}).to_string(),
        r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#.to_string(),
    ];
    assert!(session.handle_batch(&batch).is_err());
    let out = session.handle_json(&batch[1]).unwrap();
    let out: Value = serde_json::from_str(&out[0]).unwrap();
    assert_eq!(
        out["error"]["code"], -32002,
        "batch left a duplicate pending ID"
    );

    let mut session = JsonSession::new(Limits::default());
    let batch = [
        r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"exit"}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}"#.to_string(),
    ];
    let out = session.handle_batch(&batch).unwrap();
    let first: Value = serde_json::from_str(&out[0][0]).unwrap();
    assert_eq!(
        first["error"]["code"], -32002,
        "a cancellation after exit affected earlier work"
    );
}

#[test]
fn shutdown_stops_pending_publications_and_background_requests() {
    let mut session = JsonSession::new(Limits::default());
    session.defer_publications(true);
    session
        .handle_json(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        )
        .unwrap();
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
        .unwrap();
    session.handle_json(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///wire/pending.jai","version":1,"languageId":"jai","text":"main :: ( {"}}}"#).unwrap();
    assert!(session.publications_pending());
    session
        .handle_json(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#)
        .unwrap();
    assert!(!session.publications_pending());
    assert!(!session.compile_pending());
    assert!(!session.has_background_work());
    assert!(session.flush_publications().unwrap().is_empty());
    assert!(session.warm_pending().unwrap().is_empty());
    assert!(session.background_work().unwrap().is_empty());
    session
        .handle_json(r#"{"jsonrpc":"2.0","id":99,"result":null}"#)
        .unwrap();
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":99}}"#)
        .unwrap();
    session
        .handle_json(r#"{"jsonrpc":"2.0","method":"exit"}"#)
        .unwrap();
    assert_eq!(session.exit_status(), Some(0));
}

#[test]
fn byte_bridge_batch_bounds_retained_output_and_recovers_pending_ids() {
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#.to_string();
    let single = JsonSession::new(Limits::default())
        .handle_json(&request)
        .unwrap();
    let mut session = JsonSession::new(Limits {
        output_bytes: single[0].len() + 1,
        ..Limits::default()
    });
    let other = r#"{"jsonrpc":"2.0","id":8,"method":"unknown"}"#.to_string();
    assert!(
        session.handle_batch(&[request, other.clone()]).is_err(),
        "batch retained more than the output cap"
    );
    let next = session.handle_json(&other).unwrap();
    let next: Value = serde_json::from_str(&next[0]).unwrap();
    assert_eq!(next["error"]["code"], -32002);
}

#[test]
fn cancelled_but_unanswered_ids_remain_duplicates_within_a_batch() {
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"unknown"}"#.to_string();
    let cancel = r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}"#.to_string();
    for batch in [
        vec![request.clone(), cancel.clone(), request.clone()],
        vec![
            request.clone(),
            cancel.clone(),
            request.clone(),
            cancel.clone(),
        ],
    ] {
        let mut session = JsonSession::new(Limits::default());
        let output = session.handle_batch(&batch).unwrap();
        let first: Value = serde_json::from_str(&output[0][0]).unwrap();
        let duplicate: Value = serde_json::from_str(&output[2][0]).unwrap();
        assert_eq!(first["error"]["code"], -32800);
        assert_eq!(duplicate["error"]["code"], -32600);
        let later = session.handle_json(&request).unwrap();
        let later: Value = serde_json::from_str(&later[0]).unwrap();
        assert_eq!(
            later["error"]["code"], -32002,
            "completion must permit later ID reuse"
        );
    }
}

#[test]
fn unidentifiable_peer_responses_do_not_generate_response_loops() {
    let mut session = JsonSession::new(Limits::default());
    for input in [
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"peer parse error"}}"#,
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"peer invalid request"}}"#,
        r#"{"jsonrpc":"2.0","id":null,"result":null}"#,
    ] {
        assert!(
            session.handle_json(input).unwrap().is_empty(),
            "a response must not provoke another response"
        );
    }
    let output = session
        .handle_json(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#,
        )
        .unwrap();
    let output: Value = serde_json::from_str(&output[0]).unwrap();
    assert!(
        output.get("result").is_some(),
        "ignored response changed lifecycle"
    );
}

#[test]
fn ignored_peer_response_does_not_split_cancellation_admission() {
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"shutdown"}"#.to_owned();
    let cancel = r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}"#.to_owned();
    for response in [
        r#"{"jsonrpc":"2.0","id":null,"result":null}"#,
        r#"{"jsonrpc":"2.0","id":999,"result":null}"#,
    ] {
        let mut session = JsonSession::new(Limits::default());
        let output = session
            .handle_batch(&[request.clone(), response.into(), cancel.clone()])
            .unwrap();
        assert_eq!(output.len(), 3);
        assert!(output[1].is_empty());
        let first: Value = serde_json::from_str(&output[0][0]).unwrap();
        assert_eq!(
            first["error"]["code"], -32800,
            "peer response must not create a dispatch barrier"
        );
    }
}

#[test]
fn traced_batch_keeps_actual_dispatch_origins_through_cancellation_and_emit_failure() {
    use jai_language_server::trace::{Category, Origin, Trace, current_origin};
    use serde_json::json;
    let directory = test_support::TempDir::new("trace-batch");
    let path = directory.path().join("trace.json");
    let trace = Trace::open_new(&path, 64).unwrap();
    let mut session = JsonSession::new(Limits::default());
    let bodies = [
        json!({"jsonrpc":"2.0","id":7,"method":"unknown"}),
        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":7}}),
        json!({"jsonrpc":"2.0","id":8,"method":"unknown"}),
    ];
    let frames: Vec<_> = bodies
        .iter()
        .map(|body| {
            trace
                .frame(Category::Request, body.to_string().len())
                .unwrap()
        })
        .collect();
    let messages = bodies
        .into_iter()
        .map(|body| serde_json::from_value(body).unwrap())
        .collect();
    let mut groups = Vec::new();
    session
        .dispatch_messages_traced(messages, &trace, frames.clone(), |messages| {
            groups.push((current_origin(), messages));
            Ok(())
        })
        .unwrap();
    assert_eq!(groups.len(), 3);
    for (at, (origin, _)) in groups.iter().enumerate() {
        assert_eq!(*origin, Origin::Frame(frames[at].ordinal));
    }
    assert!(
        groups[1].1.is_empty(),
        "cancellation is applied during admission"
    );
    let cancelled = serde_json::to_value(&groups[0].1[0]).unwrap();
    assert_eq!(cancelled["error"]["code"], -32800);
    assert_eq!(current_origin(), Origin::None);

    let frame = trace.frame(Category::Request, 1).unwrap();
    let request =
        serde_json::from_value(json!({"jsonrpc":"2.0","id":9,"method":"unknown"})).unwrap();
    let error = session.dispatch_messages_traced(vec![request], &trace, vec![frame], |_| {
        assert_eq!(current_origin(), Origin::Frame(frame.ordinal));
        Err(jai_language_server::ProtocolError::OutputLimit)
    });
    assert!(matches!(
        error,
        Err(jai_language_server::ProtocolError::OutputLimit)
    ));
    assert_eq!(current_origin(), Origin::None);
    trace.finish_output(true).unwrap();
    let output: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for frame in frames.into_iter().chain([frame]) {
        for stage in ["dispatch_start", "dispatch_end"] {
            assert_eq!(
                output["records"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["stage"] == stage
                        && row["origin"] == json!({"kind":"frame","ordinal":frame.ordinal}))
                    .count(),
                1
            );
        }
    }
}

#[test]
fn one_traced_dispatch_can_emit_multiple_messages_with_one_origin() {
    use jai_language_server::trace::{Category, Origin, Trace, current_origin};
    use serde_json::json;
    let directory = test_support::TempDir::new("trace-many-output");
    let path = directory.path().join("trace.json");
    let trace = Trace::open_new(&path, 16).unwrap();
    let mut session = JsonSession::new(Limits::default());
    for body in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///one.jai","languageId":"jai","version":1,"text":"one :: 1;"}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///two.jai","languageId":"jai","version":1,"text":"two :: 2;"}}}),
    ] {
        session.handle_json(&body.to_string()).unwrap();
    }
    let body = json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///one.jai"}}});
    let frame = trace
        .frame(Category::Notification, body.to_string().len())
        .unwrap();
    let mut groups = 0;
    session
        .dispatch_messages_traced(
            vec![serde_json::from_value(body).unwrap()],
            &trace,
            vec![frame],
            |messages| {
                groups += 1;
                assert_eq!(
                    messages.len(),
                    2,
                    "remaining document publication and closed-document clearing publication"
                );
                for message in messages {
                    assert_eq!(current_origin(), Origin::Frame(frame.ordinal));
                    let value = serde_json::to_value(message).unwrap();
                    assert_eq!(value["method"], "textDocument/publishDiagnostics");
                }
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(groups, 1);
    assert_eq!(current_origin(), Origin::None);
    assert_eq!(trace.finish_output(true).unwrap().unwrap().recorded, 3);
}
