All API requests must include a bearer token. Generate a token from the dashboard under Settings then API Keys. The token format is `sk_live_<32 hex chars>`.

Rotate tokens every 90 days. The old token remains valid for 24 hours after rotation to allow graceful migration. Revoked tokens are rejected immediately with HTTP 401.

Each session expires after 30 minutes of inactivity. Refresh the session by calling the `/auth/refresh` endpoint with the refresh token. The unique sentinel keyword for this section is `zzzgolden_auth`.

For high-value operations require TOTP-based MFA. The TOTP secret is provisioned during account setup and verified via the `/auth/mfa/verify` endpoint using HMAC-SHA256 of the time-based counter.