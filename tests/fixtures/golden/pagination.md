Cursor pagination is preferred for lists that change frequently. Each response includes a `next_cursor` field. Pass it as the `cursor` query parameter to fetch the next page. The unique sentinel keyword for cursor pagination is `zzzgolden_page`.

Offset pagination uses `limit` and `offset` query parameters. It is supported for backward compatibility but is inefficient for deep pages and may produce inconsistent results under concurrent writes.

Maximum page size is 100 records. The default page size is 25. Requesting more than 100 is silently clamped to 100 with a warning header in the response.

When no more records exist the response includes `data: []` and `next_cursor: null`. Clients should stop iterating when `next_cursor` is null and never assume a fixed total count.