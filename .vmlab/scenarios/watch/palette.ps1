# Prints the bounds of Demysto's windows on screen each time they change, or
# "none" when there are none: fast enough to see a resize no user should.
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Text;
using System.Runtime.InteropServices;
public static class Watch {
  [StructLayout(LayoutKind.Sequential)] struct Rect { public int L, T, R, B; }
  delegate bool Each(IntPtr window, IntPtr data);
  [DllImport("user32.dll")] static extern bool EnumWindows(Each each, IntPtr data);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr window);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr window, out Rect rect);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr window, StringBuilder text, int most);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public static string Bounds(HashSet<uint> pids) {
    var found = new List<string>();
    EnumWindows((window, data) => {
      uint pid;
      GetWindowThreadProcessId(window, out pid);
      var title = new StringBuilder(64);
      GetWindowText(window, title, title.Capacity);
      Rect r;
      // By title, which leaves out the tray icon's own small windows.
      if (pids.Contains(pid) && IsWindowVisible(window) && title.ToString() == "Demysto"
          && GetWindowRect(window, out r))
        found.Add(r.L + " " + r.T + " " + (r.R - r.L) + " " + (r.B - r.T));
      return true;
    }, IntPtr.Zero);
    return found.Count == 0 ? "none" : string.Join(" | ", found);
  }
}
'@
[void][Watch]::SetProcessDPIAware()
# Asked once: the app is running before anything is watched.
$pids = [System.Collections.Generic.HashSet[uint32]]::new()
Get-Process demysto | ForEach-Object { [void]$pids.Add([uint32]$_.Id) }
$last = $null
while ($true) {
  $now = [Watch]::Bounds($pids)
  if ($now -ne $last) {
    [Console]::Out.WriteLine($now)
    [Console]::Out.Flush()
    $last = $now
  }
}
