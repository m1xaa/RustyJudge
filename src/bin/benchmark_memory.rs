use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug)]
struct MemoryRun {
    dataset: String,
    elapsed_ms: u128,
    peak_memory_kib: u64,
    exit_code: Option<i32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let datasets = vec![
        ("small", "benchmarks/corpus/small.js"),
        ("medium", "benchmarks/corpus/medium.js"),
        ("large", "benchmarks/corpus/large.js"),
    ];

    let cli_path = cli_binary_path();

    println!("dataset,elapsed_ms,peak_memory_kib,exit_code");

    for (name, input_path) in datasets {
        let result = measure_process_memory(
            name.to_string(),
            &cli_path,
            input_path,
        )?;

        println!(
            "{},{},{},{}",
            result.dataset,
            result.elapsed_ms,
            result.peak_memory_kib,
            result
                .exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }

    Ok(())
}

fn cli_binary_path() -> PathBuf {
    if let Ok(path) = env::var("RUSTY_JUDGE_BIN") {
        return PathBuf::from(path);
    }

    #[cfg(windows)]
    {
        PathBuf::from("target/release/rusty_judge.exe")
    }

    #[cfg(not(windows))]
    {
        PathBuf::from("target/release/rusty_judge")
    }
}

fn measure_process_memory(
    dataset: String,
    cli_path: &PathBuf,
    input_path: &str,
) -> Result<MemoryRun, Box<dyn std::error::Error>> {
    let start = Instant::now();

    let mut child = Command::new(cli_path)
        .arg(input_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let pid = Pid::from_u32(child.id());
    let mut system = System::new();

    let mut peak_memory_kib = 0u64;

    loop {
        system.refresh_processes(
            ProcessesToUpdate::Some(&[pid]),
            true,
        );

        if let Some(process) = system.process(pid) {
            let memory = process.memory();

            if memory > peak_memory_kib {
                peak_memory_kib = memory;
            }
        }

        if let Some(status) = child.try_wait()? {
            return Ok(MemoryRun {
                dataset,
                elapsed_ms: start.elapsed().as_millis(),
                peak_memory_kib,
                exit_code: status.code(),
            });
        }

        thread::sleep(Duration::from_millis(5));
    }
}