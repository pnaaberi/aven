use crate::ids::WorkspaceId;
use anyhow::Result;
use aven_core::db::Database;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::cli::ContextArgs;
use crate::query::{self, TaskDependencyItem};
use crate::render::{print_json_pretty, print_multiline_block, quote};
use crate::task_render::{
    AttachmentMetadataJson, MetadataDetailJson, TaskEpicLinkJson, TaskRecurrenceJson,
    TaskRelatedJson, attachment_metadata_json, print_attachment_section, task_epic_link_json,
    task_recurrence_json, task_related_json,
};
use crate::types::Task;
use crate::workspaces::Workspace;

pub(crate) async fn cmd_context(
    database: &Database,
    workspace: &Workspace,
    blob_dir: &Path,
    args: ContextArgs,
) -> Result<()> {
    let task = database.resolve_task_ref(workspace, &args.task_ref).await?;
    let snapshot = task_context_snapshot(database, workspace, blob_dir, &task).await?;
    if args.json {
        print_json_pretty(&snapshot)?;
    } else {
        print_task_context(&snapshot, args.full);
    }
    Ok(())
}

#[derive(Serialize)]
struct TaskContextSnapshot {
    task: ContextTask,
    project: ContextProject,
    workspace: ContextWorkspace,
    labels: Vec<String>,
    metadata: BTreeMap<String, String>,
    metadata_details: Vec<MetadataDetailJson>,
    dependencies: ContextDependencies,
    related: Vec<TaskRelatedJson>,
    notes: Vec<ContextNote>,
    conflicts: Vec<ContextConflict>,
    has_conflicts: bool,
    is_blocked: bool,
    has_open_dependents: bool,
    epic_parent: Option<TaskEpicLinkJson>,
    epic_children: Vec<TaskEpicLinkJson>,
    recurrence: Option<TaskRecurrenceJson>,
    attachments: Vec<AttachmentMetadataJson>,
}

#[derive(Serialize)]
struct ContextTask {
    id: String,
    ref_suffix: String,
    display_ref: String,
    title: String,
    description: String,
    status: String,
    priority: String,
    available_at: String,
    due_on: String,
    deleted: bool,
    is_epic: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ContextProject {
    id: String,
    key: String,
    name: String,
    prefix: String,
}

#[derive(Serialize)]
struct ContextWorkspace {
    id: String,
    key: String,
    name: String,
}

#[derive(Serialize)]
struct ContextDependencies {
    depends_on_open: usize,
    depends_on_total: usize,
    blocks_open: usize,
    blocks_total: usize,
    depends_on: Vec<ContextDependencyTask>,
    blocks: Vec<ContextDependencyTask>,
}

#[derive(Serialize)]
struct ContextDependencyTask {
    id: String,
    display_ref: String,
    title: String,
    status: String,
    priority: String,
    deleted: bool,
    unresolved: bool,
    created_at: String,
}

#[derive(Serialize)]
struct ContextNote {
    id: String,
    created_at: String,
    body: String,
}

#[derive(Serialize)]
struct ContextConflict {
    field: String,
    variants: Vec<ContextConflictVariant>,
}

#[derive(Serialize)]
struct ContextConflictVariant {
    token: String,
    value: String,
}

async fn task_context_snapshot(
    database: &Database,
    workspace: &Workspace,
    blob_dir: &Path,
    task: &Task,
) -> Result<TaskContextSnapshot> {
    let display_refs = database.display_ref_context(&workspace.id).await?;
    let display_ref = display_refs.display_ref(task);
    let ref_suffix = display_refs.display_suffix(&workspace.id, &task.id);
    let detail = database
        .task_detail_with_display_refs(task, &display_refs)
        .await?;
    let labels = detail.item.labels.clone();
    let metadata = detail
        .item
        .metadata
        .iter()
        .map(|metadata| (metadata.key.clone(), metadata.value.clone()))
        .collect();
    let metadata_details = detail
        .item
        .metadata
        .iter()
        .map(|metadata| MetadataDetailJson {
            field_id: metadata.field_id.to_string(),
            key: metadata.key.clone(),
            value: metadata.value.clone(),
        })
        .collect();
    let recurrence = detail.item.recurrence.as_ref().map(task_recurrence_json);
    let epic_parent = detail.item.epic_parent.as_ref().map(task_epic_link_json);
    let epic_children = detail
        .item
        .epic_children
        .iter()
        .map(task_epic_link_json)
        .collect();
    let related = detail.related.iter().map(task_related_json).collect();
    let summary = detail.dependencies;
    let details = detail.conflicts;
    let attachments = database
        .attachment_read_items_by_task(blob_dir, &task.workspace_id, &task.id, true)
        .await?
        .into_iter()
        .map(attachment_metadata_json)
        .collect();

    let depends_on_open = summary
        .depends_on
        .iter()
        .filter(|item| item.unresolved)
        .count();
    let blocks_open = summary.blocks.iter().filter(|item| item.unresolved).count();
    let depends_on_total = summary.depends_on.len();
    let blocks_total = summary.blocks.len();
    let has_conflicts = !details.is_empty();
    let is_blocked = depends_on_open > 0;
    let has_open_dependents = blocks_open > 0;

    Ok(TaskContextSnapshot {
        task: ContextTask {
            id: task.id.to_string(),
            ref_suffix,
            display_ref,
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status.as_str().to_string(),
            priority: task.priority.as_str().to_string(),
            available_at: task.available_at.clone().unwrap_or_default(),
            due_on: task.due_on.clone().unwrap_or_default(),
            deleted: task.deleted,
            is_epic: task.is_epic,
            created_at: task.created_at.clone(),
            updated_at: task.updated_at.clone(),
        },
        project: ContextProject {
            id: task.project_id.to_string(),
            key: task.project_key.clone(),
            name: detail.project_name,
            prefix: task.project_prefix.clone(),
        },
        workspace: ContextWorkspace {
            id: workspace.id.to_string(),
            key: workspace.key.clone(),
            name: workspace.name.clone(),
        },
        labels,
        metadata,
        metadata_details,
        dependencies: ContextDependencies {
            depends_on_open,
            depends_on_total,
            blocks_open,
            blocks_total,
            depends_on: summary
                .depends_on
                .into_iter()
                .map(context_dependency_task)
                .collect(),
            blocks: summary
                .blocks
                .into_iter()
                .map(context_dependency_task)
                .collect(),
        },
        related,
        notes: detail
            .notes
            .into_iter()
            .map(|note| ContextNote {
                id: note.id,
                created_at: note.created_at,
                body: note.body,
            })
            .collect(),
        conflicts: context_conflicts(database, &task.workspace_id, details).await?,
        has_conflicts,
        is_blocked,
        has_open_dependents,
        epic_parent,
        epic_children,
        recurrence,
        attachments,
    })
}

fn context_dependency_task(item: TaskDependencyItem) -> ContextDependencyTask {
    ContextDependencyTask {
        id: item.task.id.to_string(),
        display_ref: item.display_ref,
        title: item.task.title,
        status: item.task.status.as_str().to_string(),
        priority: item.task.priority.as_str().to_string(),
        deleted: item.task.deleted,
        unresolved: item.unresolved,
        created_at: item.created_at,
    }
}

async fn context_conflicts(
    database: &Database,
    workspace_id: &WorkspaceId,
    details: Vec<query::TaskDetailConflict>,
) -> Result<Vec<ContextConflict>> {
    let mut conflicts = Vec::with_capacity(details.len());
    for detail in details {
        let local_value = database
            .conflict_display_value(workspace_id, &detail.field, &detail.local_value)
            .await?;
        let remote_value = database
            .conflict_display_value(workspace_id, &detail.field, &detail.remote_value)
            .await?;
        conflicts.push(ContextConflict {
            field: detail.field,
            variants: vec![
                ContextConflictVariant {
                    token: detail.variant_a,
                    value: local_value,
                },
                ContextConflictVariant {
                    token: detail.variant_b,
                    value: remote_value,
                },
            ],
        });
    }
    Ok(conflicts)
}

/// Newest notes that always print, so a short update keeps its predecessor.
const NOTES_FLOOR: usize = 2;
/// Character budget for note bodies printed in full, counted from the newest.
const NOTES_BUDGET: usize = 4500;
/// Older notes still print in full when all of them fit this allowance,
/// because omitting a little text costs more than it saves.
const NOTES_TAIL_ALLOWANCE: usize = 1500;
/// A single note body longer than this prints as an excerpt.
const NOTE_CEILING: usize = 8000;
/// Characters of flattened body shown for an omitted note.
const NOTE_EXCERPT_LIMIT: usize = 160;

fn print_task_context(snapshot: &TaskContextSnapshot, full: bool) {
    let display_ref = &snapshot.task.display_ref;
    println!(
        "context {} suffix={} id={} status={} priority={} deleted={} epic={} blocked={} conflicts={} blocks_open={} labels={} title={}",
        snapshot.task.display_ref,
        snapshot.task.ref_suffix,
        snapshot.task.id,
        snapshot.task.status,
        snapshot.task.priority,
        yes_no(snapshot.task.deleted),
        yes_no(snapshot.task.is_epic),
        yes_no(snapshot.is_blocked),
        yes_no(snapshot.has_conflicts),
        yes_no(snapshot.has_open_dependents),
        snapshot.labels.join(","),
        quote(&snapshot.task.title),
    );
    let mut project_line = format!(
        "project={} prefix={}",
        snapshot.project.key, snapshot.project.prefix
    );
    if snapshot.project.name != snapshot.project.key {
        project_line.push_str(&format!(" name={}", quote(&snapshot.project.name)));
    }
    println!("{project_line}");
    println!("workspace={}", snapshot.workspace.key);
    println!(
        "created={} updated={}",
        snapshot.task.created_at, snapshot.task.updated_at
    );
    let schedule: Vec<String> = [
        ("available_at", &snapshot.task.available_at),
        ("due_on", &snapshot.task.due_on),
    ]
    .into_iter()
    .filter(|(_, value)| !value.is_empty())
    .map(|(key, value)| format!("{key}={value}"))
    .collect();
    if !schedule.is_empty() {
        println!("{}", schedule.join(" "));
    }
    if let Some(recurrence) = &snapshot.recurrence {
        println!(
            "recurrence series={} slot={} rule={} timezone={} lifecycle={} outcome={} projection={}",
            recurrence.series_ref,
            recurrence.slot_on,
            quote(&recurrence.rule),
            recurrence.timezone,
            recurrence.lifecycle,
            recurrence.outcome.as_deref().unwrap_or(""),
            recurrence.projection_state,
        );
    }
    if !snapshot.task.description.is_empty() {
        print_multiline_block("description", &snapshot.task.description);
    }
    for metadata in &snapshot.metadata_details {
        println!(
            "metadata field_id={} key={}",
            metadata.field_id, metadata.key
        );
        print_multiline_block("value", &metadata.value);
    }
    print_attachment_section(&snapshot.attachments);
    let deps = &snapshot.dependencies;
    if deps.depends_on_total > 0 {
        println!(
            "depends_on open={} total={}",
            deps.depends_on_open, deps.depends_on_total
        );
    }
    for item in &deps.depends_on {
        println!(
            "- {} status={} unresolved={} title={}",
            item.display_ref,
            item.status,
            yes_no(item.unresolved),
            quote(&item.title),
        );
    }
    if deps.blocks_total > 0 {
        println!(
            "blocks open={} total={}",
            deps.blocks_open, deps.blocks_total
        );
    }
    for item in &deps.blocks {
        println!(
            "- {} status={} unresolved={} title={}",
            item.display_ref,
            item.status,
            yes_no(item.unresolved),
            quote(&item.title),
        );
    }
    if !snapshot.related.is_empty() {
        println!("related total={}", snapshot.related.len());
    }
    for related in &snapshot.related {
        println!(
            "- {} status={} deleted={} title={}",
            related.display_ref,
            related.status,
            yes_no(related.deleted),
            quote(&related.title)
        );
    }
    if let Some(parent) = &snapshot.epic_parent {
        println!(
            "epic_parent {} status={} open={} title={}",
            parent.r#ref,
            parent.status,
            yes_no(parent.open),
            quote(&parent.title)
        );
    }
    if !snapshot.epic_children.is_empty() {
        println!("epic_children total={}", snapshot.epic_children.len());
    }
    for child in &snapshot.epic_children {
        println!(
            "- {} status={} open={} title={}",
            child.r#ref,
            child.status,
            yes_no(child.open),
            quote(&child.title)
        );
    }
    print_notes(&snapshot.notes, full, display_ref);
    for conflict in &snapshot.conflicts {
        println!(
            "conflict {} field={}",
            snapshot.task.display_ref, conflict.field
        );
        for variant in &conflict.variants {
            println!("variant {}", variant.token);
            print_multiline_block("value", &variant.value);
        }
    }
}

enum NoteView<'a> {
    Full,
    Excerpt(&'a str),
    Omitted,
}

/// Chooses how each note prints. The newest notes print in full, extending
/// to older notes while they fit the budget without skipping any, because
/// recent notes usually carry the current state.
fn select_notes(notes: &[ContextNote], full: bool) -> Vec<NoteView<'_>> {
    if full {
        return notes.iter().map(|_| NoteView::Full).collect();
    }
    let chars: Vec<usize> = notes.iter().map(|note| note.body.chars().count()).collect();
    let mut first_shown = notes.len();
    let mut used = 0;
    for index in (0..notes.len()).rev() {
        let within_floor = notes.len() - index <= NOTES_FLOOR;
        if !within_floor && used + chars[index] > NOTES_BUDGET {
            break;
        }
        used += chars[index];
        first_shown = index;
    }
    if chars[..first_shown].iter().sum::<usize>() <= NOTES_TAIL_ALLOWANCE {
        first_shown = 0;
    }
    notes
        .iter()
        .enumerate()
        .map(|(index, note)| {
            if index < first_shown {
                NoteView::Omitted
            } else {
                match truncate_at_line(&note.body, NOTE_CEILING) {
                    Some(prefix) => NoteView::Excerpt(prefix),
                    None => NoteView::Full,
                }
            }
        })
        .collect()
}

/// Prints notes chronologically. When any body is incomplete, a summary line
/// comes first and a final `read=` command prints every missing body.
fn print_notes(notes: &[ContextNote], full: bool, display_ref: &str) {
    let views = select_notes(notes, full);
    let incomplete: Vec<&str> = notes
        .iter()
        .zip(&views)
        .filter(|(_, view)| !matches!(view, NoteView::Full))
        .map(|(note, _)| note.id.as_str())
        .collect();
    if !incomplete.is_empty() {
        let omitted = views
            .iter()
            .filter(|view| matches!(view, NoteView::Omitted))
            .count();
        let omitted_chars: usize = notes
            .iter()
            .zip(&views)
            .map(|(note, view)| match view {
                NoteView::Full => 0,
                NoteView::Excerpt(prefix) => note.body.chars().count() - prefix.chars().count(),
                NoteView::Omitted => note.body.chars().count(),
            })
            .sum();
        println!(
            "notes total={} shown={} omitted={omitted} omitted_chars={omitted_chars}",
            notes.len(),
            notes.len() - omitted,
        );
    }
    for (note, view) in notes.iter().zip(&views) {
        match view {
            NoteView::Omitted => println!(
                "note id={} created={} chars={} excerpt={}",
                note.id,
                note.created_at,
                note.body.chars().count(),
                quote(&note_excerpt(&note.body)),
            ),
            NoteView::Full => {
                println!("note id={} created={}", note.id, note.created_at);
                print_multiline_block("body", &note.body);
            }
            NoteView::Excerpt(prefix) => {
                println!("note id={} created={}", note.id, note.created_at);
                print_multiline_block("body", prefix);
                println!(
                    "note truncated id={} shown_chars={} total_chars={}",
                    note.id,
                    prefix.chars().count(),
                    note.body.chars().count(),
                );
            }
        }
    }
    if !incomplete.is_empty() {
        println!(
            "read={}",
            quote(&format!(
                "aven note-show {display_ref} {}",
                incomplete.join(" ")
            ))
        );
    }
}

/// Flattens whitespace so an excerpt reads past short headings.
fn note_excerpt(body: &str) -> String {
    let flat = body.split_whitespace().collect::<Vec<_>>().join(" ");
    match truncate_at_char(&flat, NOTE_EXCERPT_LIMIT) {
        Some(prefix) => format!("{prefix}..."),
        None => flat,
    }
}

/// Returns the longest prefix within `limit` characters that ends at a line
/// break, falling back to a character cut when that would drop more than half
/// the budget. Returns `None` when `value` already fits.
fn truncate_at_line(value: &str, limit: usize) -> Option<&str> {
    let prefix = truncate_at_char(value, limit)?;
    match prefix.rfind('\n') {
        Some(end) if end >= prefix.len() / 2 => Some(prefix[..end].trim_end()),
        _ => Some(prefix),
    }
}

fn truncate_at_char(value: &str, limit: usize) -> Option<&str> {
    value
        .char_indices()
        .nth(limit)
        .map(|(byte_index, _)| &value[..byte_index])
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
