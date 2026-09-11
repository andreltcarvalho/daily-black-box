# Read-only diagnostics: window classes and executable identity, never titles.
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class WindowProbe {
    public delegate bool Callback(IntPtr hwnd, IntPtr data);
    [DllImport("user32.dll")] static extern bool EnumWindows(Callback callback, IntPtr data);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr hwnd, StringBuilder text, int length);
    public static string[] Inspect() {
        var rows = new List<string>();
        EnumWindows((hwnd, data) => {
            uint pid; GetWindowThreadProcessId(hwnd, out pid);
            if (IsWindowVisible(hwnd)) {
                var name = new StringBuilder(256); GetClassName(hwnd, name, name.Capacity);
                rows.Add(pid.ToString() + "|" + name.ToString());
            }
            return true;
        }, IntPtr.Zero);
        return rows.ToArray();
    }
}
'@
foreach ($row in [WindowProbe]::Inspect()) {
    $parts = $row.Split('|', 2)
    $process = Get-Process -Id ([int]$parts[0]) -ErrorAction SilentlyContinue
    if ($process -and $process.ProcessName -match 'msrdc|WindowsApp|Windows365|RdClient|chrome') {
        [pscustomobject]@{ Process = $process.ProcessName; WindowClass = $parts[1]; Executable = $process.Path }
    }
}
