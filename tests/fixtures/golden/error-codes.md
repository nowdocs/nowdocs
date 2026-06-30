All errors return a JSON body with `error.code` `error.message` and `error.docs_url` fields. The HTTP status code reflects the error category. The unique sentinel keyword for the error format is `zzzgolden_errcode`.

Common 4xx codes are 400 bad request 401 unauthorized 403 forbidden 404 not found 409 conflict 422 unprocessable entity and 429 rate limited.

Common 5xx codes are 500 internal error 502 bad gateway 503 service unavailable and 504 gateway timeout. Retries are safe for 502 503 504 but not for 500.

Each error code has a stable string identifier in `error.code` such as `invalid_token` `quota_exceeded` and `resource_not_found`. Use the identifier in your error-handling logic instead of parsing the human-readable message text.