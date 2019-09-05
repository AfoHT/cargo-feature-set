use std::process::Command;
use std::env;
use std::io::{self, Write};

use serde::Deserialize;
use tabwriter::TabWriter;

#[derive(Debug, Deserialize)]
struct BuildPlan {
    invocations: Vec<Invocation>
}

#[derive(Debug, Deserialize)]
struct Invocation {
    package_name: String,
    package_version: String,
    program: String,
    args: Vec<String>,
}

fn cargo_buildplan() -> String {
    // Removing environment variables that might change the programs cargo plans to execute.
    // We won't execute anything, so we don't need it.
    env::remove_var("RUSTC_WRAPPER");
    env::remove_var("RUSTC");

    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("-Z").arg("unstable-options")
        .arg("build").arg("--build-plan")
        .output()
        .expect("failed to execute cargo");

    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn select_crate(krate: &Invocation) -> bool {
    krate.program == "rustc"

}

fn extract_features(args: &[String]) -> Vec<String> {
    let mut res = vec![];
    for arg in args {
        let mut s = arg.split('=');
        match s.next() {
            Some("feature") => {
                let feat = s.next().unwrap();
                let feat = feat.trim_matches('"');
                res.push(feat.to_string());
            }
            _ => {}
        }
    }
    res
}

fn main() {
    let plan = cargo_buildplan();
    let plan: BuildPlan = serde_json::from_str(&plan).expect("can't parse build plan");

    let krates = plan.invocations.into_iter().filter(select_crate).map(|krate| {
        format!("{}:{}\t{}", krate.package_name, krate.package_version, extract_features(&krate.args).join(", "))
    });

    let stdout = io::stdout();
    let handle = stdout.lock();
    let mut tw = TabWriter::new(handle);

    writeln!(&mut tw, "Crate\tFeatures").unwrap();
    writeln!(&mut tw, "=====\t========").unwrap();

    for line in krates {
        writeln!(&mut tw, "{}", line).unwrap();
    }
    tw.flush().unwrap();
}