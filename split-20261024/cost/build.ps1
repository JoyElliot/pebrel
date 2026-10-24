$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-split-20261024\cost'
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
$taskTarget = "$taskRoot\target"
$taskConfig = 'F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml'
foreach ($variant in @('a','b')) {
    foreach ($step in @('clean','build')) {
        $arguments = if ($step -eq 'clean') {
            @('clean','-p','nebula_terminal','-p','pr316-native-cost','--target-dir',$taskTarget)
        } else {
            @('build','--offline','--locked','--release','--bins','--features','images','--target-dir',$taskTarget,'--config',$taskConfig)
        }
        Write-Host "$variant cost $step"
        $process = Start-Process -FilePath $taskCargo -ArgumentList $arguments -WorkingDirectory "$taskRoot\$variant" -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\$variant\$step.stdout" -RedirectStandardError "$taskRoot\$variant\$step.stderr" -PassThru
        $taskHandle = $process.Handle
        $process.WaitForExit()
        $process.Refresh()
        if ($null -eq $process.ExitCode) { throw 'Missing native cost build exit code' }
        if ($process.ExitCode -ne 0) { exit $process.ExitCode }
    }
    New-Item -ItemType Directory -Path "$taskRoot\$variant\bin" -Force | Out-Null
    foreach ($name in @('native-cost.exe','native-memory.exe','native-validate.exe')) {
        Copy-Item -LiteralPath "$taskTarget\release\$name" -Destination "$taskRoot\$variant\bin\$name"
    }
}
Write-Host 'Cost binaries frozen; measurements have not run'
exit 0
