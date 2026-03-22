use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PartitionState {
    Up,
    Down,
    Drain,
    Inactive,
    Unknown(String),
}

impl fmt::Display for PartitionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartitionState::Up => write!(f, "up"),
            PartitionState::Down => write!(f, "down"),
            PartitionState::Drain => write!(f, "drain"),
            PartitionState::Inactive => write!(f, "inact"),
            PartitionState::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for PartitionState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "up" => PartitionState::Up,
            "down" => PartitionState::Down,
            "drain" => PartitionState::Drain,
            "inact" | "inactive" => PartitionState::Inactive,
            _ => PartitionState::Unknown(s.to_string()),
        }
    }
}

impl PartitionState {
    pub fn is_available(&self) -> bool {
        matches!(self, PartitionState::Up)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub state: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub name: String,
    pub state: PartitionState,
    pub time_limit: Option<String>,
    pub node_count: u32,
    pub nodes: Vec<String>,
    pub detailed_info: Option<PartitionDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartitionDetails {
    pub max_nodes: Option<String>,
    pub max_time: Option<String>,
    pub default_time: Option<String>,
    pub min_nodes: u32,
    pub nodes: Option<String>,
    pub node_list: Vec<String>,
    pub allow_accounts: Option<String>,
    pub allow_qos: Option<String>,
    pub default_qos: Option<String>,
    pub max_cpus_per_node: Option<String>,
    pub priority_job_factor: Option<String>,
    pub priority_tier: Option<String>,
    pub state: Option<String>,
    pub preemption_mode: Option<String>,
    pub grace_time: Option<String>,
    pub hidden: bool,
    pub disable_root_jobs: bool,
    pub exclusive_user: bool,
    pub lln: bool,
}

impl Partition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            state: PartitionState::Unknown(String::new()),
            time_limit: None,
            node_count: 0,
            nodes: Vec::new(),
            detailed_info: None,
        }
    }

    pub fn with_state(mut self, state: PartitionState) -> Self {
        self.state = state;
        self
    }

    pub fn with_time_limit(mut self, time_limit: String) -> Self {
        self.time_limit = Some(time_limit);
        self
    }

    pub fn with_node_count(mut self, count: u32) -> Self {
        self.node_count = count;
        self
    }

    pub fn with_nodes(mut self, nodes: Vec<String>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn display_name(&self) -> String {
        if self.name.len() > 15 {
            format!("{}...", &self.name[..12])
        } else {
            self.name.clone()
        }
    }

    pub fn nodes_summary(&self) -> String {
        if self.nodes.is_empty() {
            format!("{} nodes", self.node_count)
        } else {
            let node_str = self.nodes.join(",");
            if node_str.len() > 30 {
                format!("{}...", &node_str[..27])
            } else {
                node_str
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionList {
    pub partitions: Vec<Partition>,
}

impl PartitionList {
    pub fn new() -> Self {
        Self {
            partitions: Vec::new(),
        }
    }

    pub fn update(&mut self, partitions: Vec<Partition>) {
        self.partitions = partitions;
    }

    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty()
    }

    pub fn available_partitions(&self) -> Vec<&Partition> {
        self.partitions
            .iter()
            .filter(|p| p.state.is_available())
            .collect()
    }
}

impl Default for PartitionList {
    fn default() -> Self {
        Self::new()
    }
}
