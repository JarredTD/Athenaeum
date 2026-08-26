//! Platform-neutral project automation.

use std::env;
use std::process::{Command, ExitCode};

struct CargoStep {
    label: &'static str,
    arguments: &'static [&'static str],
    rustdoc_flags: bool,
}

const CHECK_STEPS: &[CargoStep] = &[
    CargoStep { label: "format", arguments: &["fmt", "--check"], rustdoc_flags: false },
    CargoStep {
        label: "check",
        arguments: &["check", "--locked", "--workspace", "--no-default-features"],
        rustdoc_flags: false,
    },
    CargoStep {
        label: "clippy",
        arguments: &[
            "clippy",
            "--locked",
            "--workspace",
            "--all-features",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        rustdoc_flags: false,
    },
    CargoStep {
        label: "documentation",
        arguments: &[
            "doc",
            "--locked",
            "--workspace",
            "--all-features",
            "--no-deps",
            "--document-private-items",
        ],
        rustdoc_flags: true,
    },
    CargoStep {
        label: "coverage",
        arguments: &[
            "llvm-cov",
            "--locked",
            "--all-features",
            "--fail-under-lines",
            "95",
            "--fail-under-regions",
            "95",
        ],
        rustdoc_flags: false,
    },
    CargoStep {
        label: "dependency audit",
        arguments: &["audit", "--file", "Cargo.lock"],
        rustdoc_flags: false,
    },
];

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("check") => run_checks(),
        _ => {
            eprintln!("Usage: cargo xtask check");
            ExitCode::FAILURE
        }
    }
}

fn run_checks() -> ExitCode {
    for step in CHECK_STEPS {
        eprintln!("Checking {}...", step.label);

        let mut command = Command::new("cargo");
        command.args(step.arguments);
        if step.rustdoc_flags {
            command.env("RUSTDOCFLAGS", "-D warnings");
        }

        match command.status() {
            Ok(status) if status.success() => {}
            Ok(status) => {
                eprintln!("{} check failed with {status}", step.label);
                return ExitCode::FAILURE;
            }
            Err(error) => {
                eprintln!("could not run {} check: {error}", step.label);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
