$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-sync-20261024'
$arguments = @('check','--locked','-p','nebula','--bin','pebrel','--tests','--no-default-features','--features','legacy-shell','--target-dir','F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml')
$process = Start-Process -FilePath 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe' -ArgumentList $arguments -WorkingDirectory "$taskRoot\main-source" -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\main-legacy-check.stdout" -RedirectStandardError "$taskRoot\main-legacy-check.stderr" -PassThru -Wait
@{ configuration='main-legacy'; exit_code=$process.ExitCode } | ConvertTo-Json | Set-Content "$taskRoot\main-legacy-check-result.json" -Encoding utf8
exit $process.ExitCode
