# provider.py for Windows, where python.exe is a Store stub. Same arguments,
# same output. Listens on localhost: a 127.0.0.1 prefix needs a URL
# reservation an unelevated user cannot make.
param([int]$Port, [int]$Status, [string]$Answer, [double]$Delay)
$ErrorActionPreference = 'Stop'
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Start()
$seen = 0
while ($listener.IsListening) {
  $context = $listener.GetContext()
  $request = $context.Request
  $raw = New-Object System.IO.MemoryStream
  if ($request.HasEntityBody) { $request.InputStream.CopyTo($raw) }
  $seen++
  [Console]::Out.WriteLine("$($request.HttpMethod) $($request.Url.AbsolutePath) #$seen $([Convert]::ToBase64String($raw.ToArray()))")
  [Console]::Out.Flush()
  $code = 200
  if ($request.HttpMethod -eq 'GET') {
    $body = @{ data = @(@{ id = 'mock-small' }, @{ id = 'mock-large' }) }
  } else {
    Start-Sleep -Milliseconds ([int]($Delay * 1000))
    if ($Status -ge 400) { $code = $Status; $body = @{ error = @{ message = $Answer } } }
    else { $body = @{ choices = @(@{ message = @{ content = $Answer } }) } }
  }
  $bytes = [Text.Encoding]::UTF8.GetBytes((ConvertTo-Json $body -Depth 5 -Compress))
  $context.Response.StatusCode = $code
  $context.Response.ContentType = 'application/json'
  $context.Response.ContentLength64 = $bytes.Length
  $context.Response.OutputStream.Write($bytes, 0, $bytes.Length)
  $context.Response.Close()
}
