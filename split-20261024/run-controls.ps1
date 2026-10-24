$ErrorActionPreference = 'Stop'
$taskRoot = 'F:\Git\pebrel\tmp\pr316-split-20261024'
$taskOld = 'F:\Git\pebrel\tmp\pr316-sync-20261024'
$tests = @(
    'gpui_shell::file_editor::document::writer_tests::independently_loaded_symlink_aliases_share_the_same_writer_permit',
    'gpui_shell::settings_pane::mobile::tests::mobile_three_states_and_manual_copy_use_the_rendered_controls',
    'gpui_shell::workspace::settings_navigation::ui_tests::density_changes_real_tab_bounds_and_sidebar_drag_pitch',
    'shell_detect::tests::wsl_prompt_command_executes_repeatedly_without_checkout_carriage_returns'
)
New-Item -ItemType Directory -Path "$taskRoot\controls" -Force | Out-Null
$results = @()
foreach ($variant in @('main','a','b','c')) {
    $binary = if ($variant -eq 'main') { "$taskOld\main-test.exe" } else { "$taskRoot\$variant\product-test.exe" }
    $source = if ($variant -eq 'main') { "$taskOld\main-source" } else { "$taskRoot\$variant\source" }
    for ($index = 0; $index -lt $tests.Length; $index++) {
        Write-Host "$variant control $index"
        $process = Start-Process -FilePath $binary -ArgumentList @($tests[$index],'--exact','--test-threads=1') -WorkingDirectory $source -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\controls\$variant-$index.stdout" -RedirectStandardError "$taskRoot\controls\$variant-$index.stderr" -PassThru
        $taskHandle = $process.Handle
        $process.WaitForExit()
        $process.Refresh()
        if ($null -eq $process.ExitCode) { throw 'Missing control process exit code' }
        $results += @{ variant=$variant; test=$tests[$index]; exit_code=$process.ExitCode }
    }
}
$results | ConvertTo-Json | Set-Content "$taskRoot\controls\results.json" -Encoding utf8
exit 0
