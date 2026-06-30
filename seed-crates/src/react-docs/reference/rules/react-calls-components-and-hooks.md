---
title: React calls Components and Hooks
---

; // ✅ Good: Only use components in JSX
}
```

```js {expectedErrors: } 
function BlogPost() 
```

If a component contains Hooks, it's easy to violate the [Rules of Hooks](/reference/rules/rules-of-hooks) when components are called directly in a loop or conditionally.

Letting React orchestrate rendering also allows a number of benefits:

* **Components become more than functions.** React can augment them with features like _local state_ through Hooks that are tied to the component's identity in the tree.
* **Component types participate in reconciliation.** By letting React call your components, you also tell it more about the conceptual structure of your tree. For example, when you move from rendering `` to the `` page, React won’t attempt to re-use them.
* **React can enhance your user experience.** For example, it can let the browser do some work between component calls so that re-rendering a large component tree doesn’t block the main thread.
* **A better debugging story.** If components are first-class citizens that the library is aware of, we can build rich developer tools for introspection in development.
* **More efficient reconciliation.** React can decide exactly which components in the tree need re-rendering and skip over the ones that don't. That makes your app faster and more snappy.

---

## Never pass around Hooks as regular values 

Hooks should only be called inside of components or Hooks. Never pass it around as a regular value.

Hooks allow you to augment a component with React features. They should always be called as a function, and never passed around as a regular value. This enables _local reasoning_, or the ability for developers to understand everything a component can do by looking at that component in isolation.

Breaking this rule will cause React to not automatically optimize your component.

### Don't dynamically mutate a Hook 

Hooks should be as "static" as possible. This means you shouldn't dynamically mutate them. For example, this means you shouldn't write higher order Hooks:

```js {expectedErrors: } 
function ChatInput() 
```

Hooks should be immutable and not be mutated. Instead of mutating a Hook dynamically, create a static version of the Hook with the desired functionality.

```js 
function ChatInput() 

function useDataWithLogging() 
```

### Don't dynamically use Hooks 

Hooks should also not be dynamically used: for example, instead of doing dependency injection in a component by passing a Hook as a value:

```js {expectedErrors: } 
function ChatInput() 
```

You should always inline the call of the Hook into that component and handle any logic in there.

```js 
function ChatInput() 

function Button() 

function useDataWithLogging() 
```

This way, `` is much easier to understand and debug. When Hooks are used in dynamic ways, it increases the complexity of your app greatly and inhibits local reasoning, making your team less productive in the long term. It also makes it easier to accidentally break the [Rules of Hooks](/reference/rules/rules-of-hooks) that Hooks should not be called conditionally. If you find yourself needing to mock components for tests, it's better to mock the server instead to respond with canned data. If possible, it's also usually more effective to test your app with end-to-end tests.

