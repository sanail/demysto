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
  $response = $context.Response
  $raw = New-Object System.IO.MemoryStream
  if ($request.HasEntityBody) { $request.InputStream.CopyTo($raw) }
  $seen++
  [Console]::Out.WriteLine("$($request.HttpMethod) $($request.Url.AbsolutePath) #$seen $([Convert]::ToBase64String($raw.ToArray()))")
  [Console]::Out.Flush()
  $code = 200
  $events = $null
  if ($request.HttpMethod -eq 'GET') {
    $body = @{ data = @(@{ id = 'mock-small' }, @{ id = 'mock-large' }) }
  } else {
    Start-Sleep -Milliseconds ([int]($Delay * 1000))
    # Matched rather than parsed: Windows PowerShell's ConvertFrom-Json refuses
    # a body past 2 MB, which a picture makes.
    $streamed = [Text.Encoding]::UTF8.GetString($raw.ToArray()) -match '"stream"\s*:\s*true'
    if ($Status -ge 400) { $code = $Status; $body = @{ error = @{ message = $Answer } } }
    elseif ($streamed) {
      $third = [int][Math]::Ceiling($Answer.Length / 3)
      $events = @(for ($i = 0; $i -lt $Answer.Length; $i += $third) {
        ConvertTo-Json @{ choices = @(@{ delta = @{ content = $Answer.Substring($i, [Math]::Min($third, $Answer.Length - $i)) } }) } -Depth 5 -Compress
      })
      $events += ConvertTo-Json @{ choices = @(@{ delta = @{}; finish_reason = 'stop' }) } -Depth 5 -Compress
      $events += '[DONE]'
    }
    else { $body = @{ choices = @(@{ message = @{ content = $Answer } }) } }
  }
  if ($events) {
    $response.ContentType = 'text/event-stream'
    $response.SendChunked = $true
    try {
      foreach ($line in $events) {
        $bytes = [Text.Encoding]::UTF8.GetBytes("data: $line`n`n")
        $response.OutputStream.Write($bytes, 0, $bytes.Length)
        $response.OutputStream.Flush()
        Start-Sleep -Milliseconds 50
      }
      $response.Close()
    } catch {
      # The key check hangs up at the headers (ADR-0008).
      $response.Abort()
    }
    continue
  }
  $bytes = [Text.Encoding]::UTF8.GetBytes((ConvertTo-Json $body -Depth 5 -Compress))
  $response.StatusCode = $code
  $response.ContentType = 'application/json'
  $response.ContentLength64 = $bytes.Length
  $response.OutputStream.Write($bytes, 0, $bytes.Length)
  $response.Close()
}
