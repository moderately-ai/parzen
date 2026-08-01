use std::{
    io::{self, Write},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    HarnessResult,
    cli::{OutputFormat, RunConfig},
    scenarios::{Operation, Scenario},
};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    pub operations_per_sample: usize,
    pub raw_ns_per_operation: Vec<f64>,
    pub min_ns: f64,
    pub median_ns: f64,
    pub mean_ns: f64,
    pub stddev_ns: f64,
    pub p90_ns: f64,
    pub p95_ns: f64,
    pub operations_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStats {
    pub best_objective: f64,
    pub simple_regret: f64,
    pub best_so_far: Vec<f64>,
    pub evaluations_to_thresholds: Vec<(f64, Option<usize>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_blocks: u64,
    pub total_bytes: u64,
    pub current_blocks: usize,
    pub current_bytes: usize,
    pub blocks_after_drop: usize,
    pub bytes_after_drop: usize,
    pub peak_blocks: usize,
    pub peak_bytes: usize,
    pub bytes_per_operation: f64,
    pub retained_blocks_after_ingest: usize,
    pub retained_bytes_after_ingest: usize,
    pub warmup_allocated_blocks: u64,
    pub warmup_allocated_bytes: u64,
    pub cycle_allocated_blocks: u64,
    pub cycle_allocated_bytes: u64,
    pub peak_rss_bytes: u64,
    pub heap_profile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub timestamp_unix_seconds: u64,
    pub git_commit: String,
    pub git_dirty: bool,
    pub rustc: String,
    pub target: String,
    pub os: String,
    pub kernel: String,
    pub architecture: String,
    pub cpu: String,
    pub cpu_governor: String,
    pub cpu_affinity: String,
    pub available_parallelism: usize,
    pub machine_label: String,
    pub uptime: String,
    pub top_processes: String,
    pub macos_compositors: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    pub schema_version: u32,
    pub backend: String,
    pub backend_version: String,
    pub scenario: Scenario,
    pub operation: Operation,
    pub supported: bool,
    pub unsupported_reason: Option<String>,
    pub config: RunConfig,
    pub semantics: Vec<String>,
    pub fixture_checksum: u64,
    pub result_checksum: u64,
    pub observations: usize,
    pub profile_operations: Option<usize>,
    pub profile_wall_seconds: Option<f64>,
    pub comparison_round: Option<usize>,
    pub invocation_order: Option<usize>,
    pub timing: Option<TimingStats>,
    pub quality: Option<QualityStats>,
    pub memory: Option<MemoryStats>,
    pub environment: Environment,
}

impl BenchmarkRecord {
    #[must_use]
    pub fn unsupported(
        backend: &str,
        version: &str,
        config: RunConfig,
        semantics: Vec<String>,
        reason: String,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            backend: backend.to_owned(),
            backend_version: version.to_owned(),
            scenario: config.scenario,
            operation: config.operation,
            supported: false,
            unsupported_reason: Some(reason),
            fixture_checksum: 0,
            result_checksum: 0,
            observations: 0,
            profile_operations: None,
            profile_wall_seconds: None,
            comparison_round: None,
            invocation_order: None,
            timing: None,
            quality: None,
            memory: None,
            environment: Environment::capture(&config.machine_label),
            config,
            semantics,
        }
    }
}

impl Environment {
    #[must_use]
    pub fn capture(machine_label: &str) -> Self {
        Self {
            timestamp_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs()),
            git_commit: command_output("git", &["rev-parse", "HEAD"]),
            git_dirty: !command_output("git", &["status", "--porcelain"]).is_empty(),
            rustc: command_output("rustc", &["--version", "--verbose"]),
            target: command_output("rustc", &["-vV"])
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .unwrap_or("unknown")
                .to_owned(),
            os: std::env::consts::OS.to_owned(),
            kernel: command_output("uname", &["-srv"]),
            architecture: std::env::consts::ARCH.to_owned(),
            cpu: cpu_name(),
            cpu_governor: cpu_governor(),
            cpu_affinity: cpu_affinity(),
            available_parallelism: std::thread::available_parallelism().map_or(0, usize::from),
            machine_label: machine_label.to_owned(),
            uptime: command_output("uptime", &[]),
            top_processes: top_processes()
                .lines()
                .take(16)
                .collect::<Vec<_>>()
                .join("\n"),
            macos_compositors: if cfg!(target_os = "macos") {
                command_output("ps", &["-Ao", "pid,comm"])
                    .lines()
                    .filter(|line| {
                        let lower = line.to_ascii_lowercase();
                        lower.contains("screensaver")
                            || lower.contains("wallpaper")
                            || lower.contains("videotoolbox")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            },
        }
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |value| value.trim().to_owned())
}

fn cpu_name() -> String {
    if cfg!(target_os = "macos") {
        let value = command_output("sysctl", &["-n", "machdep.cpu.brand_string"]);
        if value != "unknown" && !value.is_empty() {
            return value;
        }
        let value = command_output("sysctl", &["-n", "hw.model"]);
        if value != "unknown" && !value.is_empty() {
            return value;
        }
    }
    if cfg!(target_os = "linux")
        && let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo")
        && let Some(name) = cpuinfo.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.trim() == "model name")
                .map(|(_, value)| value.trim().to_owned())
        })
    {
        return name;
    }
    "unknown".to_owned()
}

fn cpu_governor() -> String {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .map_or_else(|_| "unknown".to_owned(), |value| value.trim().to_owned())
}

fn cpu_affinity() -> String {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("Cpus_allowed_list:\t"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn top_processes() -> String {
    if cfg!(target_os = "linux") {
        command_output("ps", &["-Ao", "pcpu,pid,comm", "--sort=-pcpu"])
    } else {
        command_output("ps", &["-Ao", "pcpu,pid,comm", "-r"])
    }
}

#[must_use]
#[allow(unsafe_code)]
pub fn peak_rss_bytes() -> u64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: `getrusage` initializes the supplied `rusage` on a zero return code.
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    if result != 0 {
        return 0;
    }
    // SAFETY: the successful call above initialized the value.
    let usage = unsafe { usage.assume_init() };
    #[cfg(target_os = "macos")]
    {
        usage.ru_maxrss.max(0) as u64
    }
    #[cfg(not(target_os = "macos"))]
    {
        (usage.ru_maxrss.max(0) as u64).saturating_mul(1024)
    }
}

pub fn write_record(record: &BenchmarkRecord, format: OutputFormat) -> HarnessResult<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match format {
        OutputFormat::Json => serde_json::to_writer(&mut output, record)?,
        OutputFormat::Human => write_human(&mut output, record)?,
    }
    writeln!(output)?;
    Ok(())
}

fn write_human(output: &mut impl Write, record: &BenchmarkRecord) -> io::Result<()> {
    if !record.supported {
        return write!(
            output,
            "{} {} {}: unsupported ({})",
            record.backend,
            record.scenario,
            record.operation,
            record.unsupported_reason.as_deref().unwrap_or("no reason")
        );
    }
    write!(
        output,
        "{} {} {}",
        record.backend, record.scenario, record.operation
    )?;
    if let Some(timing) = &record.timing {
        write!(
            output,
            ": min {:.1} ns/op, median {:.1} ns/op, {:.1} ops/s",
            timing.min_ns, timing.median_ns, timing.operations_per_second
        )?;
    }
    if let Some(quality) = &record.quality {
        write!(
            output,
            ": best {:.6}, regret {:.6}",
            quality.best_objective, quality.simple_regret
        )?;
    }
    if let Some(memory) = &record.memory {
        write!(
            output,
            ": allocated {} bytes, peak live {} bytes, peak RSS {} bytes",
            memory.total_bytes, memory.peak_bytes, memory.peak_rss_bytes
        )?;
    }
    Ok(())
}
