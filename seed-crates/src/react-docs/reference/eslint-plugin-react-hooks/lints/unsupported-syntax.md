---
title: unsupported-syntax
---

## Rule Details 

React Compiler needs to statically analyze your code to apply optimizations. Features like `eval` and `with` make it impossible to statically understand what the code does at compile time, so the compiler can't optimize components that use them.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Using eval in component
function Component() {
  const result = eval(code); // Can't be analyzed
  return ;
}

// ❌ Using with statement
function Component() {
  with (Math) { // Changes scope dynamically
    return ;
  }
}

// ❌ Dynamic property access with eval
function Component() {
  const value = eval(`props.$`);
  return ;
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Use normal property access
function Component() {
  const value = props[propName]; // Analyzable
  return ;
}

// ✅ Use standard Math methods
function Component() {
  return ;
}
```

## Troubleshooting 

### I need to evaluate dynamic code 

You might need to evaluate user-provided code:

```js {expectedErrors: }
// ❌ Wrong: eval in component
function Calculator() {
  const result = eval(expression); // Unsafe and unoptimizable
  return Result: ;
}
```

Use a safe expression parser instead:

```js
// ✅ Better: Use a safe parser
import  from 'mathjs'; // or similar library

function Calculator() {
  const [result, setResult] = useState(null);

  const calculate = () => {
    try  catch (error) 
  };

  return (
    
      Calculate
      {result && Result: }
    
  );
}
```

