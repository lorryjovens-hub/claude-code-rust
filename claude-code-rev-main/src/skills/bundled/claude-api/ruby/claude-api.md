# Ruby Claude API

Ruby can call the Messages API directly with `Net::HTTP`.

## Minimal Example

```ruby
require 'json'
require 'net/http'
require 'uri'

uri = URI('https://api.anthropic.com/v1/messages')
http = Net::HTTP.new(uri.host, uri.port)
http.use_ssl = true

request = Net::HTTP::Post.new(uri)
request['x-api-key'] = ENV.fetch('ANTHROPIC_API_KEY')
request['anthropic-version'] = '2023-06-01'
request['content-type'] = 'application/json'
request.body = JSON.generate(
  model: '{{SONNET_ID}}',
  max_tokens: 512,
  messages: [
    {
      role: 'user',
      content: 'Write a compact incident summary for a failed deploy.'
    }
  ]
)

response = http.request(request)
raise response.body unless response.is_a?(Net::HTTPSuccess)

puts response.body
```

## Notes

- Parse the JSON response and extract text blocks from `content`.
- Set open/read timeouts on the HTTP client for production use.
- Add retry and rate-limit handling in one shared client object instead of duplicating it across jobs or controllers.
