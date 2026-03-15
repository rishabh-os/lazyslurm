FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

# Install SLURM and dependencies
RUN apt-get update && apt-get install -y \
    slurm-wlm \
    slurm-wlm-basic-plugins \
    slurmdbd \
    mariadb-server \
    munge \
    curl \
    build-essential \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Fix maria db perms
RUN mkdir -p /var/run/mysqld && chown mysql:mysql /var/run/mysqld

# Create necessary directories
RUN mkdir -p /var/spool/slurm /var/log/slurm /var/run/slurm /etc/slurm-llnl
RUN chown slurm:slurm /var/spool/slurm /var/log/slurm /var/run/slurm

# Create munge key
RUN /usr/sbin/create-munge-key
RUN chown munge:munge /etc/munge/munge.key
RUN chmod 400 /etc/munge/munge.key

# Create basic SLURM config
RUN echo "ClusterName=lazyslurm_dev" > /etc/slurm-llnl/slurm.conf && \
    echo "ControlMachine=slurmctld" >> /etc/slurm-llnl/slurm.conf && \
    echo "SlurmUser=root" >> /etc/slurm-llnl/slurm.conf && \
    echo "SlurmdUser=root" >> /etc/slurm-llnl/slurm.conf && \
    echo "SlurmctldPort=6817" >> /etc/slurm-llnl/slurm.conf && \
    echo "SlurmdPort=6818" >> /etc/slurm-llnl/slurm.conf && \
    echo "StateSaveLocation=/var/spool/slurm" >> /etc/slurm-llnl/slurm.conf && \
    echo "SlurmdSpoolDir=/var/spool/slurm" >> /etc/slurm-llnl/slurm.conf && \
    echo "SwitchType=switch/none" >> /etc/slurm-llnl/slurm.conf && \
    echo "ProctrackType=proctrack/pgid" >> /etc/slurm-llnl/slurm.conf && \
    echo "TaskPlugin=task/none" >> /etc/slurm-llnl/slurm.conf && \
    echo "ReturnToService=2" >> /etc/slurm-llnl/slurm.conf && \
    echo "FastSchedule=2" >> /etc/slurm-llnl/slurm.conf && \
    echo "MaxJobCount=10000" >> /etc/slurm-llnl/slurm.conf && \
    echo "JobAcctGatherType=jobacct_gather/linux" >> /etc/slurm-llnl/slurm.conf && \
    echo "AccountingStorageType=accounting_storage/slurmdbd" >> /etc/slurm-llnl/slurm.conf && \
    echo "AccountingStorageHost=localhost" >> /etc/slurm-llnl/slurm.conf && \
    echo "AccountingStorageUser=slurm" >> /etc/slurm-llnl/slurm.conf && \
    echo "NodeName=slurmctld CPUs=5 State=UNKNOWN" >> /etc/slurm-llnl/slurm.conf && \
    echo "PartitionName=debug Nodes=slurmctld Default=YES MaxTime=INFINITE State=UP" >> /etc/slurm-llnl/slurm.conf

# Create slurmdbd config
RUN echo "AuthType=auth/munge" > /etc/slurm-llnl/slurmdbd.conf && \
    echo "DbdAddr=localhost" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "DbdHost=localhost" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "SlurmUser=slurm" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "DebugLevel=4" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "LogFile=/var/log/slurm/slurmdbd.log" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "PidFile=/var/run/slurm/slurmdbd.pid" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "StorageType=accounting_storage/mysql" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "StorageHost=localhost" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "StoragePass=slurm" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "StorageUser=slurm" >> /etc/slurm-llnl/slurmdbd.conf && \
    echo "StorageLoc=slurm_acct_db" >> /etc/slurm-llnl/slurmdbd.conf && \
    chown slurm:slurm /etc/slurm-llnl/slurmdbd.conf && \
    chmod 600 /etc/slurm-llnl/slurmdbd.conf

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Startup script
CMD bash -c " \
    service mysql start && \
    sleep 2 && \
    mysql -e \"CREATE USER IF NOT EXISTS 'slurm'@'localhost' IDENTIFIED BY 'slurm';\" && \
    mysql -e \"GRANT ALL PRIVILEGES ON *.* TO 'slurm'@'localhost';\" && \
    mysql -e \"FLUSH PRIVILEGES;\" ; \
    service munge start && \
    slurmdbd && \
    sleep 3 && \
    sacctmgr add cluster lazyslurm_dev -i || true ; \
    slurmctld -D & \
    slurmd -D & \
    sleep 5 ; \
    scontrol update NodeName=slurmctld State=IDLE || true ; \
    echo 'SLURM ready! Your project is at /workspace' ; \
    echo 'Run: cargo run' ; \
    echo 'Submit test: sbatch --wrap=\"sleep 60\" --job-name=test' ; \
    tail -f /dev/null"