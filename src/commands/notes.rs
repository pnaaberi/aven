use anyhow::{Result, bail};
use aven_core::db::Database;
use serde::Serialize;

use crate::cli::{NoteArgs, NoteDeleteArgs, NoteShowArgs};
use crate::input::read_required_text;
use crate::render::{changed_text, print_json_pretty, print_multiline_block};
use crate::workspaces::Workspace;

pub(crate) async fn cmd_note(
    database: &Database,
    workspace: &Workspace,
    args: NoteArgs,
) -> Result<()> {
    let task = database.resolve_task_ref(workspace, &args.task_ref).await?;
    let body = read_required_text(args.text, args.file.as_deref(), args.stdin, "note")?;
    let outcome = database.add_note(workspace, &task.id, body).await?;
    let display_refs = database.display_ref_context(&workspace.id).await?;
    println!(
        "noted {} note={}",
        display_refs.display_ref(&task),
        outcome.note_id
    );
    Ok(())
}

pub(crate) async fn cmd_note_delete(
    database: &Database,
    workspace: &Workspace,
    args: NoteDeleteArgs,
) -> Result<()> {
    let task = database.resolve_task_ref(workspace, &args.task_ref).await?;
    let outcome = database
        .delete_note(workspace, &task.id, &args.note_id)
        .await?;
    let display_refs = database.display_ref_context(&workspace.id).await?;
    println!(
        "deleted-note {} note={} changed={}",
        display_refs.display_ref(&task),
        outcome.note_id,
        changed_text(outcome.changed),
    );
    Ok(())
}

#[derive(Serialize)]
struct NoteJson<'a> {
    id: &'a str,
    created_at: &'a str,
    body: &'a str,
}

/// Prints the requested notes in task order, so a batch read from `context`
/// returns only the bodies it omitted.
pub(crate) async fn cmd_note_show(
    database: &Database,
    workspace: &Workspace,
    args: NoteShowArgs,
) -> Result<()> {
    let task = database.resolve_task_ref(workspace, &args.task_ref).await?;
    let display_refs = database.display_ref_context(&workspace.id).await?;
    let detail = database
        .task_detail_with_display_refs(&task, &display_refs)
        .await?;
    let missing: Vec<&str> = args
        .note_ids
        .iter()
        .filter(|id| !detail.notes.iter().any(|note| &note.id == *id))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        bail!(
            "error unknown-note task={} note_ids={}",
            display_refs.display_ref(&task),
            missing.join(",")
        );
    }
    let notes = detail
        .notes
        .iter()
        .filter(|note| args.note_ids.contains(&note.id));
    if args.json {
        let notes: Vec<NoteJson> = notes
            .map(|note| NoteJson {
                id: &note.id,
                created_at: &note.created_at,
                body: &note.body,
            })
            .collect();
        return print_json_pretty(&notes);
    }
    for note in notes {
        println!("note id={} created={}", note.id, note.created_at);
        print_multiline_block("body", &note.body);
    }
    Ok(())
}
