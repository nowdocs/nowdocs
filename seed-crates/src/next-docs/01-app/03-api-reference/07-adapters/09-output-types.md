---
title: Output Types
description: Reference for all build output types exposed to adapters.
---

The `outputs` object contains arrays of build output types:

- `outputs.pages`: React pages from the `pages/` directory
- `outputs.pagesApi`: API routes from `pages/api/`
- `outputs.appPages`: React pages from the `app/` directory
- `outputs.appRoutes`: API and metadata routes from `app/`
- `outputs.prerenders`: ISR-enabled routes and static prerenders
- `outputs.staticFiles`: Static assets and auto-statically optimized pages
- `outputs.middleware`: Middleware function (if present)

> **Note:** When `config.output` is set to `'export'`, only `outputs.staticFiles` is populated. All other arrays (`pages`, `appPages`, `pagesApi`, `appRoutes`, `prerenders`) will be empty since the entire application is exported as static files.

For any route output with `runtime: 'edge'`, `edgeRuntime` is included and contains the canonical entry metadata for invoking that output in your edge runtime. Note that the Edge Runtime is [deprecated](/docs/messages/edge-runtime-deprecated).

## Pages (`outputs.pages`)

React pages from the `pages/` directory:

```typescript
{
  type: 'PAGES'
  id: string           // Route identifier
  filePath: string     // Path to the built file
  pathname: string     // URL pathname
  sourcePage: string   // Original source file path in pages/ directory
  runtime: 'nodejs' | 'edge'
  assets: Record  // Traced dependencies (key: relative path from repo root, value: absolute path)
  wasmAssets?: Record  // Bundled wasm files (key: name, value: absolute path)
  edgeRuntime?: 
  config: 
}
```

## API Routes (`outputs.pagesApi`)

API routes from `pages/api/`:

```typescript
{
  type: 'PAGES_API'
  id: string           // Route identifier
  filePath: string     // Path to the built file
  pathname: string     // URL pathname
  sourcePage: string   // Original relative source file path
  runtime: 'nodejs' | 'edge'
  assets: Record  // Traced dependencies (key: relative path from repo root, value: absolute path)
  wasmAssets?: Record  // Bundled wasm files (key: name, value: absolute path)
  edgeRuntime?: 
  config: 
}
```

## App Pages (`outputs.appPages`)

React pages from the `app/` directory:

```typescript
{
  type: 'APP_PAGE'
  id: string           // Route identifier
  filePath: string     // Path to the built file
  pathname: string     // URL pathname. Includes .rsc suffix for RSC routes
  sourcePage: string   // Original relative source file path
  runtime: 'nodejs' | 'edge' // Runtime the route is built for
  assets: Record  // Traced dependencies (key: relative path from repo root, value: absolute path)
  wasmAssets?: Record  // Bundled wasm files (key: name, value: absolute path)
  edgeRuntime?: 
  config: 
}
```

## App Routes (`outputs.appRoutes`)

API and metadata routes from the `app/` directory:

```typescript
{
  type: 'APP_ROUTE'
  id: string           // Route identifier
  filePath: string     // Path to the built file
  pathname: string     // URL pathname
  sourcePage: string   // Original relative source file path
  runtime: 'nodejs' | 'edge' // Runtime the route is built for
  assets: Record  // Traced dependencies (key: relative path from repo root, value: absolute path)
  wasmAssets?: Record  // Bundled wasm files (key: name, value: absolute path)
  edgeRuntime?: 
  config: 
}
```

## Prerenders (`outputs.prerenders`)

ISR-enabled routes and static prerenders:

```typescript
{
  type: 'PRERENDER'
  id: string           // Route identifier
  pathname: string     // URL pathname
  parentOutputId: string  // ID of the source page/route
  groupId: number        // Revalidation group identifier (prerenders with same groupId revalidate together)
  pprChain?: 
  parentFallbackMode?: false | null | string  // false: no additional paths (fallback: false), null: blocking render, string: path to HTML fallback
  fallback?: 
  config: 
}
```

## Static Files (`outputs.staticFiles`)

Static assets and auto-statically optimized pages:

```typescript

```

## Middleware (`outputs.middleware`)

`middleware.ts` (`.js`/`.ts`) or `proxy.ts` (`.js`/`.ts`) function (if present):

```typescript
{
  type: 'MIDDLEWARE'
  id: string           // Route identifier
  filePath: string     // Path to the built file
  pathname: string      // Always '/_middleware'
  sourcePage: string    // Always 'middleware'
  runtime: 'nodejs' | 'edge' // Runtime the route is built for
  assets: Record  // Traced dependencies (key: relative path from repo root, value: absolute path)
  wasmAssets?: Record  // Bundled wasm files (key: name, value: absolute path)
  edgeRuntime?: 
  config: {
    maxDuration?: number  // Maximum duration of the route in seconds
    preferredRegion?: string | string[]  // Preferred deployment region (deprecated)
    env?: Record  // Environment variables (edge runtime only)
    matchers?: Array<>
  }
}
```
