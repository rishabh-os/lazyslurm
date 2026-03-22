use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

use crate::models::{Job, JobState, Partition, PartitionDetails, PartitionList, PartitionState};

pub struct SlurmParser;

impl SlurmParser {
    pub fn parse_squeue_output(output: &str) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("JOBID") {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let job_id = parts[0].trim().to_string();
                let name = parts[1].trim().to_string();
                let user = parts[2].trim().to_string();
                let state = JobState::from(parts[3].trim());

                let mut job = Job::new(job_id.clone(), name, user, state);

                // Parse array job ID if present (e.g., "23673084_5" -> array_job_id=23673084, task_id=5)
                if job_id.contains('_') {
                    let array_parts: Vec<&str> = job_id.split('_').collect();
                    if array_parts.len() == 2 {
                        job.array_job_id = Some(array_parts[0].to_string());
                        job.array_task_id = array_parts[1].parse().ok();
                    }
                }

                // Additional fields if present
                if parts.len() > 4 {
                    job.time_used = Some(parts[4].trim().to_string());
                }
                if parts.len() > 5 {
                    job.node_list = Some(parts[5].trim().to_string());
                }
                if parts.len() > 6 {
                    job.partition = parts[6].trim().to_string();
                }

                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    pub fn parse_scontrol_output(output: &str) -> Result<HashMap<String, String>> {
        let mut fields = HashMap::new();

        // scontrol output format: "Key=Value Key2=Value2 ..."
        // Values can be quoted and contain spaces
        let re = Regex::new(r"(\w+)=([^\s]+(?:\s+[^\s=]+)*)")?;

        for line in output.lines() {
            for cap in re.captures_iter(line) {
                let key = cap[1].to_string();
                let value = cap[2].trim_matches('"').to_string();
                fields.insert(key, value);
            }
        }

        Ok(fields)
    }

    pub fn enhance_job_with_scontrol_data(job: &mut Job, scontrol_fields: HashMap<String, String>) {
        if let Some(submit_time) = scontrol_fields.get("SubmitTime") {
            job.submit_time = Self::parse_slurm_time(submit_time);
        }

        if let Some(start_time) = scontrol_fields.get("StartTime") {
            job.start_time = Self::parse_slurm_time(start_time);
        }

        if let Some(end_time) = scontrol_fields.get("EndTime") {
            job.end_time = Self::parse_slurm_time(end_time);
        }

        if let Some(working_dir) = scontrol_fields.get("WorkDir") {
            job.working_dir = Some(working_dir.clone());
        }

        if let Some(std_out) = scontrol_fields.get("StdOut") {
            job.std_out = Some(std_out.clone());
        }

        if let Some(std_err) = scontrol_fields.get("StdErr") {
            job.std_err = Some(std_err.clone());
        }

        if let Some(nodes) = scontrol_fields.get("NumNodes") {
            job.nodes = nodes.parse().ok();
        }

        if let Some(cpus) = scontrol_fields.get("NumCPUs") {
            job.cpus = cpus.parse().ok();
        }

        if let Some(memory) = scontrol_fields.get("MinMemoryNode") {
            job.memory = Some(memory.clone());
        }

        if let Some(reason) = scontrol_fields.get("Reason") {
            job.reason = Some(reason.clone());
        }

        if let Some(exit_code) = scontrol_fields.get("ExitCode") {
            // Exit code format is usually "0:0" where first is exit code, second is signal
            if let Some(code) = exit_code.split(':').next() {
                job.exit_code = code.parse().ok();
            }
        }

        if let Some(time_limit) = scontrol_fields.get("TimeLimit") {
            job.time_limit = Some(time_limit.clone());
        }
    }

    fn parse_slurm_time(time_str: &str) -> Option<DateTime<Utc>> {
        // SLURM time formats: "2024-01-15T10:19:13" or "2024-01-15T10:19:13.123"
        // Sometimes also "Unknown" or "None" for jobs that haven't started
        if time_str == "Unknown" || time_str == "None" || time_str.is_empty() {
            return None;
        }

        // Try parsing with seconds
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") {
            return Some(dt.and_utc());
        }

        // Try parsing with microseconds
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S%.f") {
            return Some(dt.and_utc());
        }

        None
    }

    pub fn get_job_log_paths(job: &Job) -> Vec<String> {
        let mut paths = Vec::new();

        // Primary: Use the actual StdOut path from scontrol if available
        if let Some(std_out) = &job.std_out {
            paths.push(std_out.clone());
        }

        // Secondary: Use StdErr if different
        if let Some(std_err) = &job.std_err
            && Some(std_err) != job.std_out.as_ref()
        {
            paths.push(std_err.clone());
        }

        // Fallback: Common SLURM default patterns in working directory
        if let Some(work_dir) = &job.working_dir {
            paths.push(format!("{}/slurm-{}.out", work_dir, job.job_id));
            paths.push(format!("{}/slurm-{}.err", work_dir, job.job_id));
        } else {
            // If no working directory known, try current directory
            paths.push(format!("slurm-{}.out", job.job_id));
            paths.push(format!("slurm-{}.err", job.job_id));
        }

        // Additional fallback: Check /tmp for logs (common in dev environments)
        paths.push(format!("/tmp/slurm-{}.out", job.job_id));
        paths.push(format!("/tmp/slurm-{}.err", job.job_id));

        paths
    }

    pub fn get_stdout_path(job: &Job) -> Option<String> {
        if let Some(std_out) = &job.std_out {
            return Some(std_out.clone());
        }
        if let Some(work_dir) = &job.working_dir {
            let path = format!("{}/slurm-{}.out", work_dir, job.job_id);
            if std::fs::metadata(&path).is_ok() {
                return Some(path);
            }
        }
        let path = format!("/tmp/slurm-{}.out", job.job_id);
        if std::fs::metadata(&path).is_ok() {
            return Some(path);
        }
        None
    }

    pub fn get_stderr_path(job: &Job) -> Option<String> {
        if let Some(std_err) = &job.std_err {
            return Some(std_err.clone());
        }
        if let Some(work_dir) = &job.working_dir {
            let path = format!("{}/slurm-{}.err", work_dir, job.job_id);
            if std::fs::metadata(&path).is_ok() {
                return Some(path);
            }
        }
        let path = format!("/tmp/slurm-{}.err", job.job_id);
        if std::fs::metadata(&path).is_ok() {
            return Some(path);
        }
        None
    }

    pub fn parse_sacct_output(output: &str) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            assert!(parts.len() == 15);

            let job_id = parts[0].trim();
            if job_id.is_empty() || job_id.contains(".batch") || job_id.contains(".extern") {
                continue;
            }

            let name = parts[1].trim().to_string();
            let user = parts[2].trim().to_string();
            let state = JobState::from_sacct_state(parts[3].trim());

            let mut job = Job::new(job_id.to_string(), name, user, state);

            if job_id.contains('_') {
                let array_parts: Vec<&str> = job_id.split('_').collect();
                if array_parts.len() == 2 {
                    job.array_job_id = Some(array_parts[0].to_string());
                    job.array_task_id = array_parts[1].parse().ok();
                }
            }

            job.start_time = Self::parse_slurm_time(parts[4].trim());
            job.end_time = Self::parse_slurm_time(parts[5].trim());
            job.time_used = Some(parts[6].trim().to_string());

            let exit_code = parts[7].trim();
            if let Some(code) = exit_code.split(':').next() {
                job.exit_code = code.parse().ok();
            }

            job.node_list = Some(parts[8].trim().to_string());

            job.cpus = parts[9].trim().parse().ok();

            job.memory = Some(parts[10].trim().to_string());

            job.partition = parts[11].trim().to_string();
            job.submit_time = Self::parse_slurm_time(parts[12].trim());
            job.reason = Some(parts[13].trim().to_string());
            let _ = parts[14]; // Trailing field from pipe delimiter

            jobs.push(job);
        }

        Ok(jobs)
    }

    pub fn parse_sinfo_output(output: &str) -> Result<PartitionList> {
        let mut partitions: Vec<Partition> = Vec::new();
        let mut seen_partitions: HashMap<String, usize> = HashMap::new();

        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("PARTITION") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let name = parts[0].trim().to_string();
                let state_str = parts[1].trim();
                let time_limit = parts[2].trim().to_string();
                let node_count: u32 = parts[3].trim().parse().unwrap_or(0);

                let mut nodes: Vec<String> = Vec::new();
                if parts.len() > 5 {
                    let node_list_str = parts[5..].join(" ");
                    nodes = Self::parse_node_list(&node_list_str);
                }

                let state = PartitionState::from(state_str);

                if let Some(idx) = seen_partitions.get(&name) {
                    partitions[*idx].node_count += node_count;
                    partitions[*idx].nodes.extend(nodes);
                } else {
                    let partition = Partition::new(name.clone())
                        .with_state(state)
                        .with_time_limit(time_limit)
                        .with_node_count(node_count)
                        .with_nodes(nodes);
                    let idx = partitions.len();
                    seen_partitions.insert(name, idx);
                    partitions.push(partition);
                }
            }
        }

        Ok(PartitionList { partitions })
    }

    fn parse_node_list(node_list_str: &str) -> Vec<String> {
        let mut nodes = Vec::new();
        let cleaned = node_list_str.trim();

        if cleaned.is_empty() {
            return nodes;
        }

        for part in cleaned.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.contains('[') && part.contains("..") {
                if let Some((prefix, range)) = part.split_once('[') {
                    let range = range.trim_end_matches(']');
                    if range.contains("..")
                        && let Some((start_str, end_str)) = range.split_once("..")
                        && let (Ok(start), Ok(end)) =
                            (start_str.parse::<u32>(), end_str.parse::<u32>())
                    {
                        for i in start..=end {
                            nodes.push(format!("{}{}", prefix, i));
                        }
                    }
                }
            } else if part.contains('[') {
                if let Some((prefix, rest)) = part.split_once('[') {
                    let items = rest.trim_end_matches(']');
                    for item in items.split(',') {
                        nodes.push(format!("{}{}", prefix, item.trim()));
                    }
                }
            } else {
                nodes.push(part.to_string());
            }
        }

        nodes
    }

    pub fn parse_scontrol_partition_details(output: &str) -> Option<PartitionDetails> {
        let mut details = PartitionDetails::default();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let pairs: Vec<(&str, &str)> = line
                .split_whitespace()
                .filter_map(|s| {
                    let mut iter = s.splitn(2, '=');
                    match (iter.next(), iter.next()) {
                        (Some(k), Some(v)) => Some((k, v)),
                        _ => None,
                    }
                })
                .collect();

            for (key, value) in pairs {
                match key {
                    "PartitionName" => {
                        details.node_list = vec![value.to_string()];
                    }
                    "MaxNodes" => {
                        details.max_nodes = Some(value.to_string());
                    }
                    "MaxTime" => {
                        details.max_time = Some(value.to_string());
                    }
                    "DefaultTime" => {
                        details.default_time = Some(value.to_string());
                    }
                    "MinNodes" => {
                        details.min_nodes = value.parse().unwrap_or(0);
                    }
                    "Nodes" => {
                        details.nodes = Some(value.to_string());
                    }
                    "AllowAccounts" => {
                        details.allow_accounts = Some(value.to_string());
                    }
                    "AllowQos" => {
                        details.allow_qos = Some(value.to_string());
                    }
                    "DefaultQOS" => {
                        details.default_qos = Some(value.to_string());
                    }
                    "MaxCPUsPerNode" => {
                        details.max_cpus_per_node = Some(value.to_string());
                    }
                    "PriorityJobFactor" => {
                        details.priority_job_factor = Some(value.to_string());
                    }
                    "PriorityTier" => {
                        details.priority_tier = Some(value.to_string());
                    }
                    "State" => {
                        details.state = Some(value.to_string());
                    }
                    "PreemptMode" => {
                        details.preemption_mode = Some(value.to_string());
                    }
                    "GraceTime" => {
                        details.grace_time = Some(value.to_string());
                    }
                    "Hidden" => {
                        details.hidden = value.to_lowercase() == "yes";
                    }
                    "DisableRootJobs" => {
                        details.disable_root_jobs = value.to_lowercase() == "yes";
                    }
                    "ExclusiveUser" => {
                        details.exclusive_user = value.to_lowercase() == "yes";
                    }
                    "LLN" => {
                        details.lln = value.to_lowercase() == "yes";
                    }
                    _ => {}
                }
            }
        }

        if details.nodes.is_some()
            || details.max_nodes.is_some()
            || details.max_time.is_some()
            || details.default_time.is_some()
        {
            Some(details)
        } else {
            None
        }
    }
}
