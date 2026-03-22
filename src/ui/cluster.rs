use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::ui::App;
use crate::{
    FocusedPanel,
    models::{Partition, PartitionList, PartitionState},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ClusterPanel {
    PartitionList,
    PartitionDetails,
}

pub fn render_cluster_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_partition_list(app, frame, chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    render_partition_details(frame, app, right_chunks[0]);
}

fn render_partition_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let focus_style = if app.focused_panel == FocusedPanel::ClusterInfo {
        Style::default().fg(app.theme.focused)
    } else {
        Style::default().fg(app.theme.unfocused)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(focus_style)
        .title("Partitions:");
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    if app.partition_list.partitions.is_empty() {
        let empty = Paragraph::new("No partitions available").alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let partitions: Vec<ListItem> = app
        .partition_list
        .partitions
        .iter()
        .map(|partition| {
            let state_color = match partition.state {
                PartitionState::Up => Color::Green,
                PartitionState::Down => Color::Red,
                PartitionState::Drain => Color::Yellow,
                PartitionState::Inactive => Color::DarkGray,
                PartitionState::Unknown(_) => Color::Gray,
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<15} ", partition.display_name()),
                    Style::default(),
                ),
                Span::styled(
                    format!("{:>5} ", partition.state),
                    Style::default().fg(state_color),
                ),
            ]))
        })
        .collect();

    let list = List::new(partitions)
        .block(Block::default())
        .highlight_style(
            Style::new()
                .bg(app.theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_partition_index));
    frame.render_stateful_widget(list, inner, &mut state);
}

fn render_partition_details(frame: &mut Frame, app: &mut App, area: Rect) {
    let focus_style = if app.focused_panel == FocusedPanel::ClusterInfo {
        Style::default().fg(app.theme.focused)
    } else {
        Style::default().fg(app.theme.unfocused)
    };

    let partition_name = app
        .partition_list
        .partitions
        .get(app.selected_partition_index)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Select a partition".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(focus_style)
        .title(format!("Partition Details ({})", partition_name));
    frame.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));

    if app.partition_list.partitions.is_empty() {
        let empty = Paragraph::new("No partition selected").alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let partition = app
        .partition_list
        .partitions
        .get(app.selected_partition_index);
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

pub fn get_selected_partition(
    partition_list: &PartitionList,
    selected_index: usize,
) -> Option<&Partition> {
    partition_list.partitions.get(selected_index)
}
