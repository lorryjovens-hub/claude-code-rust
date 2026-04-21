# Java Claude API

For Java, a simple `HttpClient` integration is enough to get started.

## Minimal Example

```java
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

public class Main {
  public static void main(String[] args) throws Exception {
    String apiKey = System.getenv("ANTHROPIC_API_KEY");
    String json = """
      {
        "model": "{{SONNET_ID}}",
        "max_tokens": 512,
        "messages": [
          {"role": "user", "content": "List three production-readiness checks."}
        ]
      }
      """;

    HttpRequest request = HttpRequest.newBuilder()
        .uri(URI.create("https://api.anthropic.com/v1/messages"))
        .header("x-api-key", apiKey)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .POST(HttpRequest.BodyPublishers.ofString(json))
        .build();

    HttpClient client = HttpClient.newHttpClient();
    HttpResponse<String> response =
        client.send(request, HttpResponse.BodyHandlers.ofString());

    System.out.println(response.body());
  }
}
```

## Notes

- Parse the response with Jackson or your preferred JSON library once you know which fields you need.
- Prefer a shared `HttpClient` and explicit timeouts.
- For server apps, log request IDs and status codes so you can diagnose throttling and malformed requests quickly.
