use anyhow::{Context, Result};
use std::process::Command;
use tokio::process::Command as TokioCommand;

pub struct SlurmCommands;

impl SlurmCommands {
    pub async fn squeue(user: Option<&str>, partition: Option<&str>) -> Result<String> {
        let mut cmd = TokioCommand::new("squeue");

        if let Some(user) = user {
            cmd.arg("-u").arg(user);
        }

        if let Some(partition) = partition {
            cmd.arg("-p").arg(partition);
        }

        cmd.arg("--format=%i,%j,%u,%t,%M,%N,%P");

        let output = cmd.output().await.context("Failed to execute squeue")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("squeue failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn scontrol_show_job(job_id: &str) -> Result<String> {
        let output = TokioCommand::new("scontrol")
            .arg("show")
            .arg("job")
            .arg(job_id)
            .output()
            .await
            .context("Failed to execute scontrol")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("scontrol show job failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn scancel(job_id: &str) -> Result<()> {
        let output = TokioCommand::new("scancel")
            .arg(job_id)
            .output()
            .await
            .context("Failed to execute scancel")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("scancel failed: {}", stderr);
        }

        Ok(())
    }

    pub async fn sacct(user: Option<&str>, partition: Option<&str>) -> Result<String> {
        let mut cmd = TokioCommand::new("sacct");

        if let Some(user) = user {
            cmd.arg("-u").arg(user);
        }

        if let Some(partition) = partition {
            cmd.arg("--partition").arg(partition);
        }

        cmd.arg("--noheader");
        cmd.arg("-p");
        cmd.arg("--format=JobID,JobName,User,State,Start,End,Elapsed,ExitCode,NNodes,NCPUs,ReqMem,Partition,Submit,Reason");

        let output = cmd.output().await.context("Failed to execute sacct")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("sacct failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn check_slurm_available() -> bool {
        Command::new("which")
            .arg("squeue")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn sinfo() -> Result<String> {
        let output = TokioCommand::new("sinfo")
            .output()
            .await
            .context("Failed to execute sinfo")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("sinfo failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn scontrol_show_partitions() -> Result<String> {
        let output = TokioCommand::new("scontrol")
            .arg("show")
            .arg("partitions")
            .output()
            .await
            .context("Failed to execute scontrol show partitions")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("scontrol show partitions failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn scontrol_show_partition(partition_name: &str) -> Result<String> {
        let output = TokioCommand::new("scontrol")
            .arg("show")
            .arg("partition")
            .arg(partition_name)
            .output()
            .await
            .context("Failed to execute scontrol show partition")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "scontrol show partition {} failed: {}",
                partition_name,
                stderr
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn sshare() -> Result<String> {
        let output = TokioCommand::new("sshare")
            .arg("-a")
            .arg("-l")
            .output()
            .await
            .context("Failed to execute sshare")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("sshare failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
