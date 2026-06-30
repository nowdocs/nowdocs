---
title: target
---

```js

```

---

## Reference 

### `target` 

Configures the React version compatibility for the compiled output.

#### Type 

```
'17' | '18' | '19'
```

#### Default value 

`'19'`

#### Valid values 

- **`'19'`**: Target React 19 (default). No additional runtime required.
- **`'18'`**: Target React 18. Requires `react-compiler-runtime` package.
- **`'17'`**: Target React 17. Requires `react-compiler-runtime` package.

#### Caveats 

- Always use string values, not numbers (e.g., `'17'` not `17`)
- Don't include patch versions (e.g., use `'18'` not `'18.2.0'`)
- React 19 includes built-in compiler runtime APIs
- React 17 and 18 require installing `react-compiler-runtime@latest`

---

## Usage 

### Targeting React 19 (default) 

For React 19, no special configuration is needed:

```js

```

The compiler will use React 19's built-in runtime APIs:

```js
// Compiled output uses React 19's native APIs
import  from 'react/compiler-runtime';
```

### Targeting React 17 or 18 

For React 17 and React 18 projects, you need two steps:

1. Install the runtime package:

```bash
npm install react-compiler-runtime@latest
```

2. Configure the target:

```js
// For React 18

// For React 17

```

The compiler will use the polyfill runtime for both versions:

```js
// Compiled output uses the polyfill
import  from 'react-compiler-runtime';
```

---

## Troubleshooting 

### Runtime errors about missing compiler runtime 

If you see errors like "Cannot find module 'react/compiler-runtime'":

1. Check your React version:
   ```bash
   npm why react
   ```

2. If using React 17 or 18, install the runtime:
   ```bash
   npm install react-compiler-runtime@latest
   ```

3. Ensure your target matches your React version:
   ```js
   
   ```

### Runtime package not working 

Ensure the runtime package is:

1. Installed in your project (not globally)
2. Listed in your `package.json` dependencies
3. The correct version (`@latest` tag)
4. Not in `devDependencies` (it's needed at runtime)

### Checking compiled output 

To verify the correct runtime is being used, note the different import (`react/compiler-runtime` for builtin, `react-compiler-runtime` standalone package for 17/18):

```js
// For React 19 (built-in runtime)
import  from 'react/compiler-runtime'
//                      ^

// For React 17/18 (polyfill runtime)
import  from 'react-compiler-runtime'
//                      ^
```