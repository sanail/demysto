# Puts the display at WIDTHxHEIGHT without saving it, or with no size back at
# the mode the user saved.
param([int]$Width, [int]$Height)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Screen {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct DevMode {
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string DeviceName;
    public short SpecVersion, DriverVersion, Size, DriverExtra;
    public int Fields, PositionX, PositionY, Orientation, FixedOutput;
    public short Color, Duplex, YResolution, TTOption, Collate;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string FormName;
    public short LogPixels;
    public int BitsPerPel, PelsWidth, PelsHeight, DisplayFlags, DisplayFrequency,
      ICMMethod, ICMIntent, MediaType, DitherType, Reserved1, Reserved2, PanningWidth, PanningHeight;
  }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern bool EnumDisplaySettings(string device, int mode, ref DevMode dm);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int ChangeDisplaySettings(ref DevMode dm, int flags);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int ChangeDisplaySettings(IntPtr none, int flags);
  public static int Restore() { return ChangeDisplaySettings(IntPtr.Zero, 0); }
  const int DM_PELSWIDTH = 0x80000, DM_PELSHEIGHT = 0x100000;
  public static int Set(int width, int height) {
    var dm = new DevMode();
    dm.Size = (short)Marshal.SizeOf(typeof(DevMode));
    EnumDisplaySettings(null, -1, ref dm);
    dm.PelsWidth = width;
    dm.PelsHeight = height;
    dm.Fields = DM_PELSWIDTH | DM_PELSHEIGHT;
    return ChangeDisplaySettings(ref dm, 0);
  }
}
'@
$result = if ($Width) { [Screen]::Set($Width, $Height) } else { [Screen]::Restore() }
if ($result -ne 0) { throw "ChangeDisplaySettings answered $result" }
