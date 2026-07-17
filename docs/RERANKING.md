# Native Cohere Reranking

nowdocs can optionally use Cohere's native v2 Rerank API to reorder the local
hybrid-retrieval candidate pool. It is disabled by default. With no reranking
configuration, retrieval remains local-only.

This is a direct Cohere integration. It requires a Cohere account and API key;
OpenRouter credentials and `cohere/...` OpenRouter model slugs are not supported
in this release.

## Configure it

Set these variables in the environment that starts `nowdocs serve` or runs
`nowdocs smoke`:

```bash
export NOWDOCS_RERANK_PROVIDER=cohere
export NOWDOCS_RERANK_MODEL=rerank-v3.5
export COHERE_API_KEY='your-cohere-api-key'
# Optional. Defaults to 2000; allowed range is 100 through 10000.
export NOWDOCS_RERANK_TIMEOUT_MS=2000
```

`NOWDOCS_RERANK_PROVIDER`, `NOWDOCS_RERANK_MODEL`, and `COHERE_API_KEY` are
required to enable reranking. The timeout is optional and measured in
milliseconds. Any partial or invalid opt-in stops `nowdocs serve` or
`nowdocs smoke` before it begins a search; it does not silently run local-only.

To disable reranking, unset the provider, model, and timeout variables:

```bash
unset NOWDOCS_RERANK_PROVIDER NOWDOCS_RERANK_MODEL NOWDOCS_RERANK_TIMEOUT_MS
```

`COHERE_API_KEY` by itself does not enable reranking, so it can remain in your
shell or credential manager for other software.

For an MCP client, these are process environment variables, not MCP tool
arguments. If your client supports an environment block for stdio servers, put
them there; otherwise launch the client from an environment that provides them.
Desktop clients often inherit a smaller environment than an interactive shell.

## Choose a model

nowdocs passes the supplied non-empty model identifier to Cohere's native v2
Rerank endpoint. Choose a model that your Cohere account can use. Common
examples are:

| Example identifier | Use when |
|---|---|
| `rerank-v3.5` | You already use this Cohere Rerank model or want an established baseline to evaluate. |
| `rerank-v4.0-fast` | Interactive latency and throughput matter most. |
| `rerank-v4.0-pro` | You prioritize ranking quality for more complex workloads. |

See Cohere's [Rerank model documentation](https://docs.cohere.com/docs/rerank)
for current availability and account terms.

## What changes at runtime

With reranking enabled, `nowdocs serve` and the CLI `nowdocs smoke` call Cohere
after local hybrid retrieval has built its candidate pool. Cohere supplies only
an ordering. nowdocs then applies its local MMR diversity ranking and its local
answer-confidence decision as usual.

Provider relevance scores are not returned to MCP clients, are not used as
answer-confidence scores, and do not appear in evaluator output. The internal
`retrieval_eval` example currently uses the local traced pipeline, so it is a
local baseline rather than an evaluation of the configured Cohere reranker.

If the remote request fails because of a connection problem, timeout,
authentication failure, rate limit, upstream error, or invalid response,
nowdocs uses the unchanged local ranking path for that search. A failed remote
request may already have transmitted its input. Reranking is optional and does
not promise a quality improvement.

## Data sent to Cohere

Each remote request contains:

- The search query.
- At most 40 candidate documents from the local hybrid-retrieval pool.
- For each candidate, a sanitized heading path and chunk text, combined and
  capped at 8,192 UTF-8 bytes.

The request does not include local chunk IDs, embeddings, cache paths,
filesystem paths, or Cohere relevance scores. nowdocs sends `COHERE_API_KEY`
only in the API's required `Authorization` header. It does not persist the key
or include it in logs, MCP output, or evaluator output.

Read Cohere's [Privacy Policy](https://cohere.com/privacy) before enabling this
integration. The project-wide network-activity disclosure is in
[PRIVACY.md](PRIVACY.md).

## Troubleshooting

| Symptom | What to check |
|---|---|
| `unsupported rerank provider` | Set `NOWDOCS_RERANK_PROVIDER=cohere` exactly. |
| A model or key is reported missing | Set both `NOWDOCS_RERANK_MODEL` and `COHERE_API_KEY` in the process that launches nowdocs. |
| The timeout is rejected | Use an integer from 100 through 10000 for `NOWDOCS_RERANK_TIMEOUT_MS`. |
| A desktop client does not rerank | Check that its stdio server process receives the variables; exporting them only in a separate terminal may not be enough. |
| Cohere is unavailable or rejects a request | Search continues using local ranking. Check your Cohere account, model access, network, and rate limits. |

To verify that the same configuration is applied outside your MCP client, run a
real smoke search in that environment:

```bash
nowdocs smoke nextjs "middleware matcher configuration"
```

This can make a request under your Cohere account. Use the disable command
above to compare the local-only path.
