# Shows the picture at $Path over the whole primary display, stretched to it,
# until killed. An ordinary window, not a topmost one: Demysto's windows open
# over it.
param([string]$Path, [string]$Title)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$form = New-Object Windows.Forms.Form
$form.Text = $Title
$form.FormBorderStyle = 'None'
$form.StartPosition = 'Manual'
$form.Bounds = [Windows.Forms.Screen]::PrimaryScreen.Bounds
$form.BackgroundImage = [Drawing.Image]::FromFile($Path)
$form.BackgroundImageLayout = 'Stretch'
[Windows.Forms.Application]::Run($form)
