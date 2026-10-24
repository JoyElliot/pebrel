$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
Set-Location 'F:\Git\pebrel\tmp\pr316-split-20261024\a\source'
$process = Start-Process -FilePath 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe' -ArgumentList @('test','--locked','-p','nebula','--test','file_line_budget','--features','gpui-shell','--target-dir','F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current','--config','F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml') -WindowStyle Hidden -RedirectStandardOutput 'F:\Git\pebrel\tmp\pr316-split-20261024\a\file-budget.stdout' -RedirectStandardError 'F:\Git\pebrel\tmp\pr316-split-20261024\a\file-budget.stderr' -PassThru
$taskHandle = $process.Handle
$process.WaitForExit()
$process.Refresh()
if ($null -eq $process.ExitCode) { throw 'No exit code for the file-budget tests' }
exit $process.ExitCode
