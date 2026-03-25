use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "praxis")]
#[command(about = "AI workflow observability and control plane — instrument, ingest, triage, and act")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the structured thinking workflow on a problem
    Think {
        /// The problem to think through
        problem: String,

        /// Force streaming output even when stdout is not a terminal
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "no_stream")]
        stream: bool,

        /// Disable streaming output
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "stream")]
        no_stream: bool,

        /// Read local repository context from the current working directory
        #[arg(long, action = ArgAction::SetTrue)]
        repo: bool,

        /// Show streamed reasoning separately from final answer content
        #[arg(long, action = ArgAction::SetTrue)]
        show_reasoning: bool,
    },

    /// Ingest session logs from AI tools into the local database
    Ingest {
        /// Only ingest from this source: opencode, claude-code, praxis, ccusage
        #[arg(long)]
        source: Option<String>,

        /// Override OpenCode data directory
        #[arg(long)]
        opencode_dir: Option<PathBuf>,

        /// Override Claude Code projects directory
        #[arg(long)]
        claude_code_dir: Option<PathBuf>,
    },

    /// List recent AI runs across all sources
    Runs {
        /// Filter by source (opencode, claude-code, praxis)
        #[arg(long)]
        source: Option<String>,

        /// Filter by model (substring match)
        #[arg(long)]
        model: Option<String>,

        /// Filter by project (substring match)
        #[arg(long)]
        project: Option<String>,

        /// Show runs since this date (YYYY-MM-DD or Nd for N days ago)
        #[arg(long)]
        since: Option<String>,

        /// Maximum number of runs to show
        #[arg(long, default_value = "20")]
        limit: usize,

        /// Output as JSON
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
    },

    /// Show aggregate AI usage statistics
    Stats {
        /// Group by: day, model, project, source
        #[arg(long)]
        by: Option<String>,

        /// Show stats since this date (YYYY-MM-DD or Nd for N days ago)
        #[arg(long)]
        since: Option<String>,

        /// Show stats until this date (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Output as JSON
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
    },

    // ─── Phase 3: Control Plane ───────────────────────────────────────────────

    /// Auto-discover workflows from OpenCode commands and systemd timers
    Discover,

    /// List and inspect registered workflows
    Workflows {
        /// Show status (last run, health) for each workflow
        #[arg(long, action = ArgAction::SetTrue)]
        status: bool,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Show only stale workflows (past their expected frequency)
        #[arg(long, action = ArgAction::SetTrue)]
        stale: bool,

        #[command(subcommand)]
        command: Option<WorkflowsSubcommand>,
    },

    /// Collect new signals from workflow output directories
    Collect,

    /// Show collected signals
    Signals {
        /// Show only untriaged signals
        #[arg(long, action = ArgAction::SetTrue)]
        untriaged: bool,

        /// Filter by workflow name
        #[arg(long)]
        workflow: Option<String>,
    },

    /// Run LLM triage on untriaged signals
    Triage,

    /// Show and manage Todoist tasks
    Tasks {
        #[command(subcommand)]
        command: Option<TasksSubcommand>,
    },

    /// Run collect + triage + status in one step
    Sync,

    /// Show prioritised morning status dashboard
    Status {
        /// Show only urgent (ACT NOW) items
        #[arg(long, action = ArgAction::SetTrue)]
        urgent: bool,

        /// Output as JSON
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum WorkflowsSubcommand {
    /// Show details for a specific workflow
    Show {
        /// Workflow name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum TasksSubcommand {
    /// Add a new task to Todoist
    Add {
        /// Task content
        content: String,

        /// Due date (natural language, e.g. "tomorrow", "next Monday")
        #[arg(long)]
        due: Option<String>,

        /// Priority 1–4 (4 = urgent)
        #[arg(long)]
        priority: Option<u8>,
    },

    /// Complete (close) a task
    Done {
        /// Todoist task ID
        id: String,
    },
}
