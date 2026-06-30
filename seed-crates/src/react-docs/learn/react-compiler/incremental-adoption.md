---
title: Incremental Adoption
---

## Why Incremental Adoption? 

React Compiler is designed to optimize your entire codebase automatically, but you don't have to adopt it all at once. Incremental adoption gives you control over the rollout process, letting you test the compiler on small parts of your app before expanding to the rest.

Starting small helps you build confidence in the compiler's optimizations. You can verify that your app behaves correctly with compiled code, measure performance improvements, and identify any edge cases specific to your codebase. This approach is especially valuable for production applications where stability is critical.

Incremental adoption also makes it easier to address any Rules of React violations the compiler might find. Instead of fixing violations across your entire codebase at once, you can tackle them systematically as you expand compiler coverage. This keeps the migration manageable and reduces the risk of introducing bugs.

By controlling which parts of your code get compiled, you can also run A/B tests to measure the real-world impact of the compiler's optimizations. This data helps you make informed decisions about full adoption and demonstrates the value to your team.

## Approaches to Incremental Adoption 

There are three main approaches to adopt React Compiler incrementally:

1. **Babel overrides** - Apply the compiler to specific directories
2. **Opt-in with "use memo"** - Only compile components that explicitly opt in
3. **Runtime gating** - Control compilation with feature flags

All approaches allow you to test the compiler on specific parts of your application before full rollout.

## Directory-Based Adoption with Babel Overrides 

Babel's `overrides` option lets you apply different plugins to different parts of your codebase. This is ideal for gradually adopting React Compiler directory by directory.

### Basic Configuration 

Start by applying the compiler to a specific directory:

```js
// babel.config.js
module.exports = {
  plugins: [
    // Global plugins that apply to all files
  ],
  overrides: [
    {
      test: './src/modern/**/*.',
      plugins: [
        'babel-plugin-react-compiler'
      ]
    }
  ]
};
```

### Expanding Coverage 

As you gain confidence, add more directories:

```js
// babel.config.js
module.exports = {
  plugins: [
    // Global plugins
  ],
  overrides: [
    {
      test: ['./src/modern/**/*.', './src/features/**/*.'],
      plugins: [
        'babel-plugin-react-compiler'
      ]
    },
    {
      test: './src/legacy/**/*.',
      plugins: [
        // Different plugins for legacy code
      ]
    }
  ]
};
```

### With Compiler Options 

You can also configure compiler options per override:

```js
// babel.config.js
module.exports = {
  plugins: [],
  overrides: [
    {
      test: './src/experimental/**/*.',
      plugins: [
        ['babel-plugin-react-compiler', ]
      ]
    },
    {
      test: './src/production/**/*.',
      plugins: [
        ['babel-plugin-react-compiler', ]
      ]
    }
  ]
};
```

## Opt-in Mode with "use memo" 

For maximum control, you can use `compilationMode: 'annotation'` to only compile components and hooks that explicitly opt in with the `"use memo"` directive.

### Annotation Mode Configuration 

```js
// babel.config.js
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ],
  ],
};
```

### Using the Directive 

Add `"use memo"` at the beginning of functions you want to compile:

```js
function TodoList() {
  "use memo"; // Opt this component into compilation

  const sortedTodos = todos.slice().sort();

  return (
    
      
    
  );
}

function useSortedData(data) 
```

With `compilationMode: 'annotation'`, you must:
- Add `"use memo"` to every component you want optimized
- Add `"use memo"` to every custom hook
- Remember to add it to new components

This gives you precise control over which components are compiled while you evaluate the compiler's impact.

## Runtime Feature Flags with Gating 

The `gating` option enables you to control compilation at runtime using feature flags. This is useful for running A/B tests or gradually rolling out the compiler based on user segments.

### How Gating Works 

The compiler wraps optimized code in a runtime check. If the gate returns `true`, the optimized version runs. Otherwise, the original code runs.

### Gating Configuration 

```js
// babel.config.js
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', {
      gating: ,
    }],
  ],
};
```

### Implementing the Feature Flag 

Create a module that exports your gating function:

```js
// ReactCompilerFeatureFlags.js
export function isCompilerEnabled() 
```

## Troubleshooting Adoption 

If you encounter issues during adoption:

1. Use `"use no memo"` to temporarily exclude problematic components
2. Check the [debugging guide](/learn/react-compiler/debugging) for common issues
3. Fix Rules of React violations identified by the ESLint plugin
4. Consider using `compilationMode: 'annotation'` for more gradual adoption

## Next Steps 

- Read the [configuration guide](/reference/react-compiler/configuration) for more options
- Learn about [debugging techniques](/learn/react-compiler/debugging)
- Check the [API reference](/reference/react-compiler/configuration) for all compiler options