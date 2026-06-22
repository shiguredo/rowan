use std::{
    env,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("error: {}", err);
        std::process::exit(1)
    }
}

pub struct Section {
    name: &'static str,
    start: Instant,
}

pub struct CargoToml {
    path: PathBuf,
    contents: String,
}

impl CargoToml {
    pub fn version(&self) -> Result<&str> {
        self.get("version")
    }

    fn get(&self, field: &str) -> Result<&str> {
        for line in self.contents.lines() {
            let words = line.split_ascii_whitespace().collect::<Vec<_>>();
            match words.as_slice() {
                [n, "=", v, ..] if n.trim() == field => {
                    assert!(v.starts_with('"') && v.ends_with('"'));
                    return Ok(&v[1..v.len() - 1]);
                }
                _ => (),
            }
        }
        Err(format!("can't find `{}` in {}", field, self.path.display()).into())
    }

    pub fn publish(&self) -> Result<()> {
        let token = env::var("CRATES_IO_TOKEN").unwrap_or("no token".to_string());
        let mut cmd = Command::new("cargo");
        cmd.arg("publish").arg("--token").arg(&token);
        if let Some(dry_run) = dry_run() {
            cmd.arg(dry_run);
        }
        run(&mut cmd)
    }

    pub fn publish_all(&self, dirs: &[&str]) -> Result<()> {
        let token = env::var("CRATES_IO_TOKEN").unwrap_or("no token".to_string());
        if dry_run().is_none() {
            for &dir in dirs {
                for _ in 0..20 {
                    std::thread::sleep(Duration::from_secs(10));
                    let mut cmd = Command::new("cargo");
                    cmd.arg("publish")
                        .arg("--manifest-path")
                        .arg(format!("{}/Cargo.toml", dir))
                        .arg("--token")
                        .arg(&token)
                        .arg("--dry-run");
                    if run(&mut cmd).is_ok() {
                        break;
                    }
                }
                let mut cmd = Command::new("cargo");
                cmd.arg("publish")
                    .arg("--manifest-path")
                    .arg(format!("{}/Cargo.toml", dir))
                    .arg("--token")
                    .arg(&token);
                run(&mut cmd)?;
            }
        }
        Ok(())
    }
}

fn dry_run() -> Option<&'static str> {
    let dry_run = DRY_RUN.load(Ordering::Relaxed);
    if dry_run { Some("--dry-run") } else { None }
}

pub fn section(name: &'static str) -> Section {
    Section::new(name)
}

pub fn cargo_toml() -> Result<CargoToml> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let path = PathBuf::from(manifest_dir).join("Cargo.toml");
    let contents = std::fs::read_to_string(&path)?;
    Ok(CargoToml { path, contents })
}

static DRY_RUN: AtomicBool = AtomicBool::new(false);
pub fn set_dry_run(yes: bool) {
    DRY_RUN.store(yes, Ordering::Relaxed)
}

fn try_main() -> Result<()> {
    let subcommand = std::env::args().nth(1);
    match subcommand {
        Some(it) if it == "ci" => (),
        _ => {
            print_usage();
            return Err("invalid arguments".into());
        }
    }
    let cargo_toml = cargo_toml()?;
    {
        let _s = section("TEST");
        for &release in &[None, Some("--release")] {
            let mut cmd = Command::new("cargo");
            cmd.arg("test");
            if let Some(release) = release {
                cmd.arg(release);
            }
            cmd.arg("--workspace").arg("--").arg("--nocapture");
            run(&mut cmd)?;
        }
    }

    let version = cargo_toml.version()?;
    let tag = format!("v{}", version);

    let dry_run =
        env::var("CI").is_err() || git::has_tag(&tag)? || git::current_branch()? != "master";
    set_dry_run(dry_run);

    {
        let _s = section("PUBLISH");
        cargo_toml.publish()?;
        git::tag(&tag)?;
        git::push_tags()?;
    }
    Ok(())
}

pub mod git {
    use std::process::Command;

    use super::{Result, dry_run, read, run};

    pub fn current_branch() -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("branch").arg("--show-current");
        read(&mut cmd)
    }

    pub fn tag_list() -> Result<Vec<String>> {
        let mut cmd = Command::new("git");
        cmd.arg("tag").arg("--list");
        let tags = read(&mut cmd)?;
        let res = tags.lines().map(|it| it.trim().to_string()).collect();
        Ok(res)
    }

    pub fn has_tag(tag: &str) -> Result<bool> {
        let res = tag_list()?.iter().any(|it| it == tag);
        Ok(res)
    }

    pub fn tag(tag: &str) -> Result<()> {
        if dry_run().is_some() {
            return Ok(());
        }
        let mut cmd = Command::new("git");
        cmd.arg("tag").arg(tag);
        run(&mut cmd)
    }

    pub fn push_tags() -> Result<()> {
        // `git push --tags --dry-run` exists, but it will fail with permissions
        // error for forks.
        if dry_run().is_some() {
            return Ok(());
        }

        let mut cmd = Command::new("git");
        cmd.arg("push").arg("--tags");
        run(&mut cmd)
    }
}

fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("command failed with status {status:?}").into());
    }
    Ok(())
}

fn read(cmd: &mut Command) -> Result<String> {
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!("command failed with status {:?}", output.status).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

fn print_usage() {
    eprintln!(
        "\
Usage: cargo run -p xtask <SUBCOMMAND>

SUBCOMMANDS:
    ci
"
    )
}

impl Section {
    fn new(name: &'static str) -> Section {
        println!("::group::{}", name);
        let start = Instant::now();
        Section { name, start }
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        eprintln!("{}: {:.2?}", self.name, self.start.elapsed());
        println!("::endgroup::");
    }
}
