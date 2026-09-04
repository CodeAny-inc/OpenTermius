use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A workspace is a collection of saved terminal layouts (tabs + panes),
/// associated hosts, and metadata. Users can switch between workspaces to
/// restore different working contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub tabs: Vec<TabLayout>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub icon: Option<String>,
    /// Human-readable description of what this workspace is for.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    /// Color label for visual identification (e.g. "#3b82f6").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color: Option<String>,
    /// Hosts associated with this workspace (quick-access list).
    #[serde(default)]
    pub host_ids: Vec<Uuid>,
    /// If true, restoring the workspace auto-connects all SSH panes.
    #[serde(default)]
    pub auto_connect: bool,
}

/// A tab contains one or more panes arranged in a split layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabLayout {
    pub id: Uuid,
    pub title: String,
    /// The split tree. A leaf pane has a host_id; a split has direction + children.
    pub layout: PaneLayout,
}

/// Recursive pane layout: either a leaf (single terminal) or a split (two halves).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PaneLayout {
    /// A single terminal pane connected to a host (or local terminal).
    Pane {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        host_id: Option<Uuid>,
        /// "ssh" or "local" — defaults to "ssh" if host_id is set, "local" otherwise.
        #[serde(default = "default_terminal_type")]
        terminal_type: String,
    },
    /// A split: two sub-layouts divided either horizontally or vertically.
    Split {
        direction: SplitDirection,
        /// 0.0–1.0, the position of the divider.
        #[serde(default = "default_ratio")]
        ratio: f32,
        first: Box<PaneLayout>,
        second: Box<PaneLayout>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

fn default_terminal_type() -> String {
    "ssh".to_string()
}

fn default_ratio() -> f32 {
    0.5
}

impl Workspace {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            tabs: Vec::new(),
            icon: None,
            description: None,
            color: None,
            host_ids: Vec::new(),
            auto_connect: false,
        }
    }
}

impl TabLayout {
    pub fn new(title: impl Into<String>, layout: PaneLayout) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            layout,
        }
    }
}

impl PaneLayout {
    pub fn pane(host_id: Option<Uuid>) -> Self {
        let terminal_type = if host_id.is_some() { "ssh" } else { "local" }.to_string();
        PaneLayout::Pane {
            host_id,
            terminal_type,
        }
    }

    pub fn split(
        direction: SplitDirection,
        first: PaneLayout,
        second: PaneLayout,
    ) -> Self {
        PaneLayout::Split {
            direction,
            ratio: 0.5,
            first: Box::new(first),
            second: Box::new(second),
        }
    }
}
