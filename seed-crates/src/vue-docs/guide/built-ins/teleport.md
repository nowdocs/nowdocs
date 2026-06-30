# Teleport 

 
```

The `to` target of `
  

.modal-demo 

You can combine `
```

We could then dynamically update `isMobile`.

## Multiple Teleports on the Same Target 

A common use case would be a reusable `

```

The rendered result would be:

```html

  A
  B

```

## Deferred Teleport  

In Vue 3.5 and above, we can use the `defer` prop to defer the target resolving of a Teleport until other parts of the application have mounted. This allows the Teleport to target a container element that is rendered by Vue, but in a later part of the component tree:

```vue-html

```

Note that the target element must be rendered in the same mount / update tick with the Teleport - i.e. if the `` is only mounted a second later, the Teleport will still report an error. The defer works similarly to the `mounted` lifecycle hook.

---

**Related**

- [`` API reference](/api/built-in-components#teleport)
- [Handling Teleports in SSR](/guide/scaling-up/ssr#teleports)
