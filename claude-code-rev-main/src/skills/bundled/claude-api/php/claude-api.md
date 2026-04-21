# PHP Claude API

For PHP, start with a direct HTTPS request and wrap it later in your application service layer.

## Minimal Example

```php
<?php

$payload = [
    'model' => '{{SONNET_ID}}',
    'max_tokens' => 512,
    'messages' => [
        [
            'role' => 'user',
            'content' => 'Generate a brief API changelog entry.',
        ],
    ],
];

$ch = curl_init('https://api.anthropic.com/v1/messages');
curl_setopt_array($ch, [
    CURLOPT_POST => true,
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_HTTPHEADER => [
        'x-api-key: ' . getenv('ANTHROPIC_API_KEY'),
        'anthropic-version: 2023-06-01',
        'content-type: application/json',
    ],
    CURLOPT_POSTFIELDS => json_encode($payload),
]);

$response = curl_exec($ch);
if ($response === false) {
    throw new RuntimeException(curl_error($ch));
}

$status = curl_getinfo($ch, CURLINFO_RESPONSE_CODE);
curl_close($ch);

if ($status >= 400) {
    throw new RuntimeException("Anthropic API error: HTTP $status\n$response");
}

echo $response . PHP_EOL;
```

## Notes

- Decode successful responses with `json_decode($response, true)`.
- Treat API key loading, retries, and request logging as shared infrastructure, not per-controller code.
- If you need typed models, introduce DTOs after the request and response shapes stabilize.
