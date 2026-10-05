# Prints a line whenever the foreground window changes: ms, process, visible, title.
param([int]$Seconds = 20)
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class F {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
}
'@
$sw = [Diagnostics.Stopwatch]::StartNew(); $last = ""
"tracing"
while ($sw.ElapsedMilliseconds -lt $Seconds * 1000) {
  $h = [F]::GetForegroundWindow(); $t = New-Object Text.StringBuilder 256
  [F]::GetWindowText($h, $t, 256) | Out-Null
  $p = 0; [F]::GetWindowThreadProcessId($h, [ref]$p) | Out-Null
  $n = (Get-Process -Id $p -ErrorAction SilentlyContinue).ProcessName
  $now = "$n visible=$([F]::IsWindowVisible($h)) title='$t'"
  if ($now -ne $last) { "{0} {1}" -f $sw.ElapsedMilliseconds, $now; $last = $now }
  Start-Sleep -Milliseconds 20
}
"done"
