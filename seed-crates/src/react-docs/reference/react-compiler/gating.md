---
title: gating
---

```js
{
  gating: 
}
```

---

## Reference 

### `gating` 

Configures runtime feature flag gating for compiled functions.

#### Type 

```
 | null
```

#### Default value 

`null`

#### Properties 

- **`source`**: Module path to import the feature flag from
- **`importSpecifierName`**: Name of the exported function to import

#### Caveats 

- The gating function must return a boolean
- Both compiled and original versions increase bundle size
- The import is added to every file with compiled functions

---

## Usage 

### Basic feature flag setup 

1. Create a feature flag module:

```js
// src/utils/feature-flags.js
export function shouldUseCompiler() 
```

2. Configure the compiler:

```js
{
  gating: 
}
```

3. The compiler generates gated code:

```js
// Input
function Button(props) {
  return ;
}

// Output (simplified)
import  from './src/utils/feature-flags';

const Button = shouldUseCompiler()
  ? function Button_optimized(props) 
  : function Button_original(props) ;
```

Note that the gating function is evaluated once at module time, so once the JS bundle has been parsed and evaluated the choice of component stays static for the rest of the browser session.

---

## Troubleshooting 

### Feature flag not working 

Verify your flag module exports the correct function:

```js
// ❌ Wrong: Default export
export default function shouldUseCompiler() 

// ✅ Correct: Named export matching importSpecifierName
export function shouldUseCompiler() 
```

### Import errors 

Ensure the source path is correct:

```js
// ❌ Wrong: Relative to babel.config.js

// ✅ Correct: Module resolution path

// ✅ Also correct: Absolute path from project root

```
