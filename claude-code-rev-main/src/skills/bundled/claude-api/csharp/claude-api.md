# C# Claude API

Use the Messages API over HTTPS when you need a minimal C# integration.

## Prerequisites

- Set `ANTHROPIC_API_KEY`
- Use the Messages endpoint: `https://api.anthropic.com/v1/messages`
- Send headers:
  - `x-api-key`
  - `anthropic-version: 2023-06-01`
  - `content-type: application/json`

## Minimal Example

```csharp
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

var apiKey = Environment.GetEnvironmentVariable("ANTHROPIC_API_KEY");
using var http = new HttpClient();

http.DefaultRequestHeaders.Add("x-api-key", apiKey);
http.DefaultRequestHeaders.Add("anthropic-version", "2023-06-01");
http.DefaultRequestHeaders.Accept.Add(
    new MediaTypeWithQualityHeaderValue("application/json")
);

var payload = new
{
    model = "{{SONNET_ID}}",
    max_tokens = 512,
    messages = new[]
    {
        new
        {
            role = "user",
            content = "Summarize why prompt caching helps repeated requests."
        }
    }
};

var body = new StringContent(
    JsonSerializer.Serialize(payload),
    Encoding.UTF8,
    "application/json"
);

var response = await http.PostAsync(
    "https://api.anthropic.com/v1/messages",
    body
);
response.EnsureSuccessStatusCode();

var json = await response.Content.ReadAsStringAsync();
Console.WriteLine(json);
```

## Notes

- Read the first `content` block with `type: "text"` for normal text output.
- Reuse one `HttpClient` across requests.
- Add retries around `429` and `5xx` responses.
- For structured output, keep your instructions explicit and validate the JSON after receipt.
