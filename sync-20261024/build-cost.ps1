$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-sync-20261024\cost'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
foreach ($variant in @('main','merged')) {
    Set-Location "$taskRoot\$variant"
    $arguments = @('build','--offline','--release','--bins','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml')
    if ($variant -eq 'merged') { $arguments += @('--features','images') }
    $process = Start-Process -FilePath $taskCargo -ArgumentList $arguments -WorkingDirectory "$taskRoot\$variant" -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\$variant-build.stdout" -RedirectStandardError "$taskRoot\$variant-build.stderr" -PassThru -Wait
    if ($process.ExitCode -ne 0) { exit $process.ExitCode }
}
