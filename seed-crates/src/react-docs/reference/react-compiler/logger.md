---
title: logger
---

```js
{
  logger: {
    logEvent(filename, event) {
      console.log(`[Compiler] $: $`);
    }
  }
}
```

---

## Reference 

### `logger` 

Configures custom logging to track compiler behavior and debug issues.

#### Type 

```
 | null
```

#### Default value 

`null`

#### Methods 

- **`logEvent`**: Called for each compiler event with the filename and event details

#### Event types 

- **`CompileSuccess`**: Function successfully compiled
- **`CompileError`**: Function skipped due to errors
- **`CompileDiagnostic`**: Non-fatal diagnostic information
- **`CompileSkip`**: Function skipped for other reasons
- **`PipelineError`**: Unexpected compilation error
- **`Timing`**: Performance timing information

#### Caveats 

- Event structure may change between versions
- Large codebases generate many log entries

---

## Usage 

### Basic logging 

Track compilation success and failures:

```js
{
  logger: {
    logEvent(filename, event) {
      switch (event.kind) {
        case 'CompileSuccess': {
          console.log(`✅ Compiled: $`);
          break;
        }
        case 'CompileError': {
          console.log(`❌ Skipped: $`);
          break;
        }
        default: 
      }
    }
  }
}
```

### Detailed error logging 

Get specific information about compilation failures:

```js
{
  logger: {
    logEvent(filename, event) {
      if (event.kind === 'CompileError') {
        console.error(`\nCompilation failed: $`);
        console.error(`Reason: $`);

        if (event.detail.description) {
          console.error(`Details: $`);
        }

        if (event.detail.loc) {
          const  = event.detail.loc.start;
          console.error(`Location: Line $, Column $`);
        }

        if (event.detail.suggestions) 
      }
    }
  }
}
```

