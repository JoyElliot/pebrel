$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-sync-20261024'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
$results = @()
foreach ($mode in @('gpui','legacy')) {
    $features = if ($mode -eq 'gpui') { @('--features','gpui-shell') } else { @('--no-default-features','--features','legacy-shell') }
    $arguments = @('check','--locked','-p','nebula','--bin','pebrel','--tests','--target-dir','F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml') + $features
    $process = Start-Process -FilePath $taskCargo -ArgumentList $arguments -WorkingDirectory "$taskRoot\source" -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\$mode-check.stdout" -RedirectStandardError "$taskRoot\$mode-check.stderr" -PassThru -Wait
    $results += @{ configuration=$mode; exit_code=$process.ExitCode }
}
$results | ConvertTo-Json | Set-Content "$taskRoot\product-check-results.json" -Encoding utf8
if ($results | Where-Object { $_.exit_code -ne 0 }) { exit 1 }
