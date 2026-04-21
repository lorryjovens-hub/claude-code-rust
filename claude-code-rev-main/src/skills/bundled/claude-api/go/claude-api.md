# Go Claude API

Go works well with the raw Messages API using `net/http`.

## Minimal Example

```go
package main

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"os"
)

func main() {
	body := []byte(`{
	  "model": "{{SONNET_ID}}",
	  "max_tokens": 512,
	  "messages": [
	    {"role": "user", "content": "Draft a concise deployment checklist."}
	  ]
	}`)

	req, err := http.NewRequest(
		http.MethodPost,
		"https://api.anthropic.com/v1/messages",
		bytes.NewReader(body),
	)
	if err != nil {
		panic(err)
	}

	req.Header.Set("x-api-key", os.Getenv("ANTHROPIC_API_KEY"))
	req.Header.Set("anthropic-version", "2023-06-01")
	req.Header.Set("content-type", "application/json")

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		panic(err)
	}
	defer res.Body.Close()

	out, err := io.ReadAll(res.Body)
	if err != nil {
		panic(err)
	}

	fmt.Println(string(out))
}
```

## Notes

- Create a reusable `http.Client` with timeouts instead of relying on `DefaultClient` in production.
- Parse the JSON response into structs once your schema is stable.
- Retry `429`, `500`, `529`, and transient network errors with backoff.
- Keep the request builder isolated so you can reuse it for tool use, prompt caching, or files later.
