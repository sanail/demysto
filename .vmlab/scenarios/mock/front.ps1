# Prints the process whose window is in the foreground.
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class F {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
}
'@
$p = 0; [F]::GetWindowThreadProcessId([F]::GetForegroundWindow(), [ref]$p) | Out-Null
(Get-Process -Id $p -ErrorAction SilentlyContinue).ProcessName
