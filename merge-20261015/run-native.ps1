param([ValidateSet('Core','Full','Editor','MainCore')][string]$Mode)
$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-conflict-20261015'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
$taskTarget = 'F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current'
$taskConfig = 'F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml'
Set-Location "$taskRoot\source"
switch ($Mode) {
 'Core' { & $taskCargo test --locked -p nebula_terminal --all-targets --target-dir $taskTarget --config $taskConfig -- --test-threads=4 }
 'Full' { & "$taskRoot\merged-test.exe" --test-threads=4 }
 'MainCore' { Set-Location "$taskRoot\main-source"; & $taskCargo test --locked -p nebula_terminal --lib --target-dir "$taskRoot\main-core-target" --config $taskConfig -- --test-threads=4 }
 'Editor' { & "$taskRoot\source\scripts\tests\run_completion_e2e.ps1" -TestBinary "$taskRoot\merged-test.exe" -OutputDirectory "$taskRoot\native-editor" -Case editor -RuntimeDirectory 'C:\Users\Joy\AppData\Local\Programs\Pebrel\runtime' -PowerShellProgram 'C:\Program Files\PowerShell\7\pwsh.exe' }
}
exit $LASTEXITCODE
