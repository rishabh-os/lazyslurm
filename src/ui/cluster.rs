use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::models::{Partition, PartitionList, PartitionState};
use crate::ui::Theme;

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterPanel {
    PartitionList,
    PartitionDetails,
}

pub fn render_cluster_view(
    frame: &mut Frame,
    partition_list: &PartitionList,
    user_limits: &crate::models::UserLimits,
    selected_partition_index: usize,
    focused_panel: ClusterPanel,
    theme: &Theme,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_partition_list(
        frame,
        partition_list,
        selected_partition_index,
        focused_panel == ClusterPanel::PartitionList,
        theme,
        chunks[0],
    );

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .spacing(1)
        .split(chunks[1]);

    render_partition_details(
        frame,
        partition_list,
        selected_partition_index,
        focused_panel == ClusterPanel::PartitionDetails,
        theme,
        right_chunks[0],
    );

    render_user_limits(frame, user_limits, right_chunks[1]);
}

fn render_partition_list(
    frame: &mut Frame,
    partition_list: &PartitionList,
    selected_index: usize,
    is_focused: bool,
    theme: &Theme,
    area: Rect,
) {
    let focus_style = if is_focused {
        Style::default().fg(theme.focused)
    } else {
        Style::default().fg(theme.unfocused)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(focus_style)
        .title("Partitions:");
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    if partition_list.partitions.is_empty() {
        let empty = Paragraph::new("No partitions available").alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = partition_list
        .partitions
        .iter()
        .enumerate()
        .map(|(idx, partition)| {
            let state_color = match partition.state {
                PartitionState::Up => Color::Green,
                PartitionState::Down => Color::Red,
                PartitionState::Drain => Color::Yellow,
                PartitionState::Inactive => Color::DarkGray,
                PartitionState::Unknown(_) => Color::Gray,
            };

            let time_limit = partition.time_limit.as_deref().unwrap_or("--");
            let node_count = partition.node_count;

            let style = if idx == selected_index {
                Style::new()
                    .bg(theme.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<15} ", partition.display_name()), style),
                Span::styled(format!("{:>5} ", partition.state), style.fg(state_color)),
                Span::styled(format!("{:>6} nodes ", node_count), style),
                Span::styled(format!("{:>10}", time_limit), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(Block::default());

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected_index));
    frame.render_stateful_widget(list, inner, &mut state);
}

fn render_partition_details(
    frame: &mut Frame,
    partition_list: &PartitionList,
    selected_index: usize,
    is_focused: bool,
    theme: &Theme,
    area: Rect,
) {
    let focus_style = if is_focused {
        Style::default().fg(theme.focused)
    } else {
        Style::default().fg(theme.unfocused)
    };

    let partition_name = partition_list
        .partitions
        .get(selected_index)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Select a partition".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(focus_style)
        .title(format!("Partition Details ({})", partition_name));
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    if partition_list.partitions.is_empty() {
        let empty = Paragraph::new("No partition selected").alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let partition = partition_list.partitions.get(selected_index);
    let text = match partition {
        Some(p) => format_partition_details(p),
        None => "Select a partition to view details".to_string(),
    };

    let details = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(details, inner);
}

fn format_partition_details(partition: &Partition) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Name: {}", partition.name));
    lines.push(format!("State: {}", partition.state));

    if let Some(ref time_limit) = partition.time_limit {
        lines.push(format!("Time Limit: {}", time_limit));
    }

    lines.push(format!("Node Count: {}", partition.node_count));

    if let Some(ref details) = partition.detailed_info {
        if let Some(ref max_nodes) = details.max_nodes {
            lines.push(format!("Max Nodes: {}", max_nodes));
        }

        if let Some(ref max_time) = details.max_time {
            lines.push(format!("Max Time: {}", max_time));
        }

        if let Some(ref default_time) = details.default_time {
            lines.push(format!("Default Time: {}", default_time));
        }

        lines.push(format!("Min Nodes: {}", details.min_nodes));

        if let Some(ref nodes) = details.nodes {
            lines.push(format!("Nodes: {}", nodes));
        }

        if let Some(ref allow_accounts) = details.allow_accounts {
            lines.push(format!("Allow Accounts: {}", allow_accounts));
        }

        if let Some(ref allow_qos) = details.allow_qos {
            lines.push(format!("Allow QoS: {}", allow_qos));
        }

        if let Some(ref default_qos) = details.default_qos {
            lines.push(format!("Default QoS: {}", default_qos));
        }

        if let Some(ref max_cpus) = details.max_cpus_per_node {
            lines.push(format!("Max CPUs/Node: {}", max_cpus));
        }

        if let Some(ref preemption) = details.preemption_mode {
            lines.push(format!("Preemption: {}", preemption));
        }

        if details.hidden {
            lines.push("Hidden: Yes".to_string());
        }

        if details.lln {
            lines.push("LLN: Yes (Least Loaded Node)".to_string());
        }
    }

    lines.join("\n")
}

fn render_user_limits(frame: &mut Frame, user_limits: &crate::models::UserLimits, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("User Limits (Fairshare):");
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    let text = format_user_limits(user_limits);
    let limits = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(limits, inner);
}

fn format_user_limits(user_limits: &crate::models::UserLimits) -> String {
    let mut lines = Vec::new();

    if let Some(ref fairshare) = user_limits.fairshare {
        lines.push(format!("Fairshare: {}", fairshare.display_fairshare()));
        lines.push(format!(
            "  Description: {}",
            fairshare.fairshare_description()
        ));
        lines.push(format!("  Account: {}", fairshare.account));

        if let Some(ref user) = fairshare.user {
            lines.push(format!("  User: {}", user));
        }

        lines.push(String::new());
        lines.push(format!(
            "Shares: {} (norm: {:.6})",
            fairshare.raw_shares, fairshare.norm_shares
        ));
        lines.push(format!(
            "Usage: {} (eff: {:.6})",
            fairshare.raw_usage, fairshare.effectv_usage
        ));
    } else if let Some(ref account) = user_limits.account {
        lines.push(format!("Account: {}", account));
        lines.push("Fairshare: Not available".to_string());
    } else {
        lines.push("User limits: Not available".to_string());
        lines.push("(Run 'sshare' for details)".to_string());
    }

    lines.join("\n")
}

pub fn get_selected_partition(
    partition_list: &PartitionList,
    selected_index: usize,
) -> Option<&Partition> {
    partition_list.partitions.get(selected_index)
}
