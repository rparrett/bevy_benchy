use crate::config::*;
use anyhow::{anyhow, bail, Context};
use argh::FromArgs;
use regex::Regex;
use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
    sync::OnceLock,
};

mod config;

const CI_CONFIG_PATH: &str = "benchy-ci-config.ron";

#[derive(FromArgs)]
/// Options for bevy_benchy
struct Args {
    /// path to config file
    #[argh(option, default = "String::from(\"benchy.toml\")")]
    config: String,
    #[argh(option)]
    /// path to git repository
    dir: String,
}

type Results = HashMap<(Bench, Commit), f32>;

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let config_str = std::fs::read_to_string(&args.config)
        .context(format!("Failed to read \"{}\"", args.config))?;
    let config: config::Config = toml::from_str(&config_str)
        .context(format!("Failed to deserialize \"{}\"", args.config))?;

    std::env::set_current_dir(&args.dir)
        .context(format!("Failed to set working directory \"{}\"", args.dir))?;

    if config.benches.is_empty() {
        bail!("At least one bench must be configured.");
    }

    if config.commits.is_empty() {
        bail!("At least one commit must be configured.");
    }

    write_ci_config(config.frames)
        .context(format!("Failed to write CI config \"{}\"", CI_CONFIG_PATH))?;

    // SAFETY: we are single-threaded
    unsafe {
        std::env::set_var("CI_TESTING_CONFIG", CI_CONFIG_PATH);
    }

    let mut results: Results = HashMap::new();

    for commit in &config.commits {
        checkout(&commit.commit)
            .context(format!("Failed to checkout commit \"{}\"", commit.commit))?;
        apply_patches().context("Failed to apply patches.")?;

        for bench in &config.benches {
            let mut args = vec![
                "run",
                "--example",
                &bench.example,
                "--release",
                "--features",
                "bevy_ci_testing",
                "--",
            ];
            args.extend(bench.example_args.iter().map(|s| s.as_str()));

            println!("Building and running {:?}..", bench);

            let output = Command::new("cargo").args(&args).output()?;
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            let fps = get_fps(&String::from_utf8_lossy(&output.stderr))?;
            results.insert((bench.clone(), commit.clone()), fps);
        }
    }

    print_markdown_table(&results, &config);

    Ok(())
}

fn write_ci_config(frames: u32) -> anyhow::Result<()> {
    std::fs::write(CI_CONFIG_PATH, format!("(events: [({}, AppExit)])", frames))?;

    Ok(())
}

fn checkout(commit: &str) -> anyhow::Result<()> {
    Command::new("git").args(["restore", "."]).status()?;
    Command::new("git").args(["checkout", commit]).status()?;
    Command::new("cargo").args(["update"]).status()?;

    Ok(())
}

fn apply_patches() -> anyhow::Result<()> {
    let patches = [
        include_str!("../patches/average-all-frames.patch"),
        include_str!("../patches/more-logs.patch"),
    ];

    for patch in patches {
        let mut child = Command::new("git")
            .arg("apply")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(mut child_stdin) = child.stdin.take() {
            child_stdin.write_all(patch.as_bytes())?;
        }

        let _ = child.wait_with_output()?;
    }

    Ok(())
}

fn print_markdown_table(results: &Results, config: &Config) {
    print!("||");
    for bench in &config.benches {
        print!("{}|", bench.label);
    }
    println!();

    print!("|-|");
    for _ in &config.benches {
        print!("-|");
    }
    println!();

    for (i, commit) in config.commits.iter().enumerate() {
        print!("|{}|", commit.label.as_ref().unwrap_or(&commit.commit));

        for bench in &config.benches {
            let fps = results.get(&(bench.clone(), commit.clone())).unwrap();
            let first = results
                .get(&(bench.clone(), config.commits[0].clone()))
                .unwrap();

            print!("{:.2}", fps);

            if i > 0 {
                let frac = (fps - first) / first;
                let sign = if frac > 0. { "+" } else { "" };
                let sym = if frac.abs() < 0.01 {
                    "🟡"
                } else if frac < 0. {
                    "🔴"
                } else {
                    "🟢"
                };

                print!(" {}{} {:.1}%", sym, sign, frac * 100.);
            }

            print!("|");
        }
        println!();
    }
    println!();
}

fn get_fps(output: &str) -> anyhow::Result<f32> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"fps.*?avg ([\d\.]+)").unwrap());

    let (_, [fps]) = re
        .captures_iter(output)
        .map(|c| c.extract())
        .last()
        .ok_or(anyhow!("No fps line in log output."))?;

    let parsed = fps.parse::<f32>()?;

    Ok(parsed)
}
