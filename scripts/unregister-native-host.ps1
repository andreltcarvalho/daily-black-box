param([string]$HostExecutable = (Join-Path $PSScriptRoot '..\src-tauri\target\release\caixa-preta-native-host.exe'))
$ErrorActionPreference = 'Stop'
$manifestPath = Join-Path ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($HostExecutable))) 'caixa-preta-host.json'
$key = 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\local.caixapreta.dodia'
if (Test-Path -LiteralPath $key) {
    $current = (Get-Item -LiteralPath $key).GetValue('')
    if ($current -eq $manifestPath) {
        Remove-Item -LiteralPath $key
        if (Test-Path -LiteralPath $manifestPath) { Remove-Item -LiteralPath $manifestPath }
        Write-Output 'Registro da ponte removido. Dados de atividade preservados.'
    } else { Write-Output 'Outra instalação usa o registro da ponte; registro preservado.' }
}
