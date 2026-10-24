$ErrorActionPreference = 'Stop'
$taskRoot = 'F:\Git\pebrel\tmp\pr316-sync-20261024\cost'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
foreach ($variant in @('main','merged')) {
    $arguments = @('build','--offline','--release','--bin','native-validate','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml')
    if ($variant -eq 'merged') { $arguments += @('--features','images') }
    $process = Start-Process -FilePath $taskCargo -ArgumentList $arguments -WorkingDirectory "$taskRoot\$variant" -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\$variant-validate-build.stdout" -RedirectStandardError "$taskRoot\$variant-validate-build.stderr" -PassThru -Wait
    if ($process.ExitCode -ne 0) { exit $process.ExitCode }
}
