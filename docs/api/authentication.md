# Authentication

The Specter API is currently open for ChessX internal use.

For rate limiting, requests are tracked by IP address:
- 100 requests per minute per IP
- Rate limit headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

Future: API keys for third-party access.
