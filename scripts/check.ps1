$ErrorActionPreference = "Stop"
$env:RUSTDOCFLAGS = "-D warnings"

$checks = @(
    @("fmt", "--check"),
    @("check", "--locked", "--no-default-features"),
    @("clippy", "--locked", "--all-features", "--all-targets", "--", "-D", "warnings"),
    @("doc", "--locked", "--all-features", "--no-deps", "--document-private-items"),
    @("llvm-cov", "--locked", "--all-features", "--fail-under-lines", "95", "--fail-under-regions", "95"),
    @("audit", "--file", "Cargo.lock")
)

foreach ($check in $checks) {
    & cargo @check
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($check -join ' ') failed with exit code $LASTEXITCODE"
    }
}
