---
title: incompatible-library
---

## Rule Details 

Some libraries use patterns that aren't supported by React. When the linter detects usages of these APIs from a [known list](https://github.com/facebook/react/blob/main/compiler/packages/babel-plugin-react-compiler/src/HIR/DefaultModuleTypeProvider.ts), it flags them under this rule. This means that React Compiler can automatically skip over components that use these incompatible APIs, in order to avoid breaking your app.

```js
// Example of how memoization breaks with these libraries
function Form() {
  const  = useForm();

  // ❌ This value will never update, even when 'name' field changes
  const name = useMemo(() => watch('name'), [watch]);

  return Name: ; // UI appears "frozen"
}
```

React Compiler automatically memoizes values following the Rules of React. If something breaks with manual `useMemo`, it will also break the compiler's automatic optimization. This rule helps identify these problematic patterns.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ react-hook-form `watch`
function Component() {
  const  = useForm();
  const value = watch('field'); // Interior mutability
  return ;
}

// ❌ TanStack Table `useReactTable`
function Component() {
  const table = useReactTable();
  // table instance uses interior mutability
  return 

### Valid 

Examples of correct code for this rule:

```js
// ✅ For react-hook-form, use `useWatch`:
function Component() {
  const  = useForm();
  const watchedValue = useWatch();

  return (
    <>
      
      Current value: 
    </>
  );
}
```

Some other libraries do not yet have alternative APIs that are compatible with React's memoization model. If the linter doesn't automatically skip over your components or hooks that call these APIs, please [file an issue](https://github.com/facebook/react/issues) so we can add it to the linter.
