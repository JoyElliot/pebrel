$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-sync-20261024'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
Set-Location "$taskRoot\main-source"
$clean = Start-Process -FilePath $taskCargo -ArgumentList @('clean','-p','nebula_terminal','--target-dir','F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml') -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\main-core-clean.stdout" -RedirectStandardError "$taskRoot\main-core-clean.stderr" -PassThru -Wait
if ($clean.ExitCode -ne 0) { exit $clean.ExitCode }
& $taskCargo test --locked -p nebula --bin pebrel --features gpui-test-support --no-run --message-format=json --target-dir 'F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current' --config 'F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml' > "$taskRoot\main-build.jsonl" 2> "$taskRoot\main-build.stderr"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$artifact = Get-Content "$taskRoot\main-build.jsonl" | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'pebrel' -and $_.profile.test -and $_.executable } | Select-Object -Last 1
if (-not $artifact) { throw 'No test executable artifact returned' }
Copy-Item -LiteralPath $artifact.executable -Destination "$taskRoot\main-test.exe"
@{ executable=$artifact.executable; sha256=(Get-FileHash "$taskRoot\main-test.exe" -Algorithm SHA256).Hash; exit_code=0 } | ConvertTo-Json | Set-Content "$taskRoot\main-build-result.json" -Encoding utf8
