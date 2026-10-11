mod common;

use common::{
    TestEnv, contains_all, contains_none, extract_attachment_id, extract_ref, ok, png_bytes,
};
use serde_json::Value;

fn seed_context(env: &TestEnv, db: &std::path::Path) -> (String, String, String) {
    ok(env.aven(db, ["label", "create", "bug"]));
    let root = extract_ref(&ok(env.aven(db, ["add", "root", "--project", "app"])));
    let middle = extract_ref(&ok(env.aven(
        db,
        [
            "add",
            "middle",
            "--project",
            "app",
            "--label",
            "bug",
            "--description",
            "details",
            "--available-at",
            "2000-01-01T00:00:00Z",
            "--due",
            "2099-01-01",
        ],
    )));
    let leaf = extract_ref(&ok(env.aven(db, ["add", "leaf", "--project", "app"])));
    ok(env.aven(db, ["dep", "add", &middle, &root]));
    ok(env.aven(db, ["dep", "add", &leaf, &middle]));
    ok(env.aven(db, ["note", &middle, "note body"]));
    (root, middle, leaf)
}

#[test]
fn context_json_contains_structured_snapshot() {
    let env = TestEnv::new();
    let db = env.db("context-json.sqlite");
    let (root, middle, leaf) = seed_context(&env, &db);

    let output = ok(env.aven(&db, ["context", &middle, "--json"]));
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["task"]["display_ref"], middle);
    assert_eq!(value["task"]["title"], "middle");
    assert_eq!(value["task"]["description"], "details");
    assert_eq!(value["task"]["available_at"], "2000-01-01T00:00:00Z");
    assert_eq!(value["task"]["due_on"], "2099-01-01");
    assert_eq!(value["task"]["deleted"], false);
    assert_eq!(value["project"]["key"], "app");
    assert_eq!(value["project"]["name"], "app");
    assert_eq!(value["workspace"]["key"], "default");
    assert_eq!(value["labels"], serde_json::json!(["bug"]));
    assert_eq!(value["dependencies"]["depends_on_total"], 1);
    assert_eq!(value["dependencies"]["blocks_total"], 1);
    assert_eq!(value["dependencies"]["depends_on"][0]["display_ref"], root);
    assert!(
        value["dependencies"]["depends_on"][0]["created_at"]
            .as_str()
            .unwrap()
            .len()
            >= 3
    );
    assert_eq!(value["dependencies"]["blocks"][0]["display_ref"], leaf);
    assert!(
        value["dependencies"]["blocks"][0]["created_at"]
            .as_str()
            .unwrap()
            .len()
            >= 3
    );
    let note_id = value["notes"][0]["id"].as_str().unwrap();
    assert!(note_id.len() >= 3);
    let text = ok(env.aven(&db, ["context", &middle]));
    contains_all(&text, &[&format!("note id={note_id} created=")]);
    assert_eq!(value["has_conflicts"], false);
    assert_eq!(value["is_blocked"], true);
    assert_eq!(value["has_open_dependents"], true);
    assert!(value["conflicts"].as_array().unwrap().is_empty());

    ok(env.aven(&db, ["delete", &middle]));
    let deleted = ok(env.aven(&db, ["context", &middle, "--json"]));
    let deleted: Value = serde_json::from_str(&deleted).unwrap();
    assert_eq!(deleted["task"]["deleted"], true);
}

#[test]
fn context_includes_attachment_metadata() {
    let env = TestEnv::new();
    let db = env.db("context-attachments.sqlite");
    let created = ok(env.aven(&db, ["add", "context attach", "--project", "app"]));
    let task_ref = extract_ref(&created);
    let image = env.path("photo.png");
    std::fs::write(&image, png_bytes(3, 2)).unwrap();
    let added = ok(env.aven(
        &db,
        [
            "attachment",
            "add",
            &task_ref,
            image.to_str().unwrap(),
            "--alt",
            "diagram",
        ],
    ));
    let attachment_id = extract_attachment_id(&added);

    let text = ok(env.aven(&db, ["context", &task_ref]));
    contains_all(&text, &["attachment attachment_id=", "has_blob=yes"]);
    contains_none(
        &text,
        &["sha256=", "filename=", "alt_text=", "photo.png", "diagram"],
    );

    let json = ok(env.aven(&db, ["context", &task_ref, "--json"]));
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["attachments"][0]["attachment_id"], attachment_id);
    assert_eq!(value["attachments"][0]["has_blob"], true);
    assert!(value["attachments"][0]["deleted_at"].is_null());
    assert!(value["attachments"][0].get("sha256").is_none());
    assert!(value["attachments"][0].get("bytes").is_none());
}

#[test]
fn context_text_omits_empty_sections() {
    let env = TestEnv::new();
    let db = env.db("context-empty-sections.sqlite");
    let task_ref = extract_ref(&ok(env.aven(&db, ["add", "plain", "--project", "app"])));

    let text = ok(env.aven(&db, ["context", &task_ref]));
    contains_all(&text, &["project=app prefix="]);
    contains_none(
        &text,
        &[
            "name=",
            "available_at=",
            "due_on=",
            "depends_on ",
            "blocks ",
            "related total=",
            "epic_children total=",
        ],
    );
}

fn note_ids_in_order(env: &TestEnv, db: &std::path::Path, task_ref: &str) -> Vec<String> {
    let json = ok(env.aven(db, ["context", task_ref, "--json"]));
    let value: Value = serde_json::from_str(&json).unwrap();
    value["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|note| note["id"].as_str().unwrap().to_string())
        .collect()
}

fn add_notes(env: &TestEnv, db: &std::path::Path, task_ref: &str, count: usize, chars: usize) {
    for index in 0..count {
        let body = format!("note {index}\n{}", "z".repeat(chars - 7));
        ok(env.aven(db, ["note", task_ref, &body]));
    }
}

#[test]
fn context_text_prints_long_description_in_full() {
    let env = TestEnv::new();
    let db = env.db("context-long-description.sqlite");
    let description = (0..300)
        .map(|line| format!("description line {line:03} {}", "x".repeat(20)))
        .collect::<Vec<_>>()
        .join("\n");
    let task_ref = extract_ref(&ok(env.aven(
        &db,
        [
            "add",
            "long",
            "--project",
            "app",
            "--description",
            &description,
        ],
    )));

    let text = ok(env.aven(&db, ["context", &task_ref]));
    contains_all(&text, &[&description]);
    contains_none(&text, &["truncated"]);
}

#[test]
fn context_text_omits_older_notes_beyond_budget() {
    let env = TestEnv::new();
    let db = env.db("context-omitted-notes.sqlite");
    let task_ref = extract_ref(&ok(env.aven(&db, ["add", "notes", "--project", "app"])));
    add_notes(&env, &db, &task_ref, 5, 1600);
    let ids = note_ids_in_order(&env, &db, &task_ref);
    let omitted = &ids[..3];

    let text = ok(env.aven(&db, ["context", &task_ref]));
    let read_args = format!("note-show {task_ref} {}", omitted.join(" "));
    contains_all(
        &text,
        &[
            "notes total=5 shown=2 omitted=3 omitted_chars=4800",
            &format!("note id={} created=", ids[3]),
            &format!("note id={} created=", ids[4]),
            &format!("read=\"aven {read_args}\""),
        ],
    );
    for id in omitted {
        assert!(
            text.lines()
                .any(|line| line.starts_with(&format!("note id={id} "))
                    && line.contains("chars=1600 excerpt=\"note ")),
            "index line for {id}:\n{text}"
        );
    }
    assert_eq!(text.matches("body<<EOF").count(), 2);
    assert!(
        text.trim_end()
            .ends_with(&format!("read=\"aven {read_args}\""))
    );

    let read = ok(env.aven(&db, read_args.split(' ').collect::<Vec<_>>()));
    assert_eq!(read.matches("body<<EOF").count(), 3);
    for id in omitted {
        contains_all(&read, &[&format!("note id={id} created=")]);
    }
    contains_none(&read, &[&ids[3], &ids[4], "context "]);

    let full = ok(env.aven(&db, ["context", &task_ref, "--full"]));
    assert_eq!(full.matches("body<<EOF").count(), 5);
    contains_none(&full, &["read=", "notes total=", "excerpt="]);
}

#[test]
fn context_text_prints_small_remainder_of_older_notes() {
    let env = TestEnv::new();
    let db = env.db("context-note-tail.sqlite");
    let task_ref = extract_ref(&ok(env.aven(&db, ["add", "notes", "--project", "app"])));
    add_notes(&env, &db, &task_ref, 4, 1400);

    let text = ok(env.aven(&db, ["context", &task_ref]));
    assert_eq!(text.matches("body<<EOF").count(), 4);
    contains_none(&text, &["read=", "notes total=", "excerpt="]);
}

#[test]
fn context_text_cuts_oversized_note() {
    let env = TestEnv::new();
    let db = env.db("context-oversized-note.sqlite");
    let task_ref = extract_ref(&ok(env.aven(&db, ["add", "big note", "--project", "app"])));
    let note = (0..200)
        .map(|line| format!("line {line:03} {}", "y".repeat(40)))
        .collect::<Vec<_>>()
        .join("\n");
    ok(env.aven(&db, ["note", &task_ref, &note]));
    let ids = note_ids_in_order(&env, &db, &task_ref);

    let text = ok(env.aven(&db, ["context", &task_ref]));
    contains_all(
        &text,
        &[
            "notes total=1 shown=1 omitted=0 omitted_chars=",
            &format!("note truncated id={} shown_chars=", ids[0]),
            &format!("total_chars={}", note.chars().count()),
            &format!("read=\"aven note-show {task_ref} {}\"", ids[0]),
            "line 000",
        ],
    );
    contains_none(&text, &["line 199"]);
    let shown = text
        .split("body<<EOF\n")
        .nth(1)
        .and_then(|rest| rest.split("\nEOF").next())
        .unwrap();
    assert!(shown.ends_with(&"y".repeat(40)), "cut at a line boundary");
}

#[test]
fn context_includes_task_metadata() {
    let env = TestEnv::new();
    let db = env.db("context-metadata.sqlite");
    let task_ref = extract_ref(&ok(env.aven(
        &db,
        [
            "add",
            "tagged",
            "--project",
            "app",
            "--metadata",
            "source=github",
        ],
    )));

    let text = ok(env.aven(&db, ["context", &task_ref]));
    contains_all(&text, &["key=source", "value<<EOF\ngithub\nEOF"]);

    let json = ok(env.aven(&db, ["context", &task_ref, "--json"]));
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["metadata"]["source"], "github");
    assert_eq!(value["metadata_details"][0]["key"], "source");
}

#[test]
fn note_show_rejects_unknown_note_ids() {
    let env = TestEnv::new();
    let db = env.db("note-show-unknown.sqlite");
    let task_ref = extract_ref(&ok(env.aven(&db, ["add", "notes", "--project", "app"])));
    ok(env.aven(&db, ["note", &task_ref, "kept"]));
    let ids = note_ids_in_order(&env, &db, &task_ref);

    let json = ok(env.aven(&db, ["note-show", &task_ref, &ids[0], "--json"]));
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value[0]["body"], "kept");

    let output = env.aven(&db, ["note-show", &task_ref, &ids[0], "MISSINGNOTEID"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    contains_all(&stderr, &["error unknown-note", "note_ids=MISSINGNOTEID"]);
}
