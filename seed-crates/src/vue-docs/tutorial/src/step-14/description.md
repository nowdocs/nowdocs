# Slots 

In addition to passing data via props, the parent component can also pass down template fragments to the child via **slots**:

```vue-html

```

```vue-html

  This is some slot content!

```

In the child component, it can render the slot content from the parent using the `` element as outlet:

```vue-html

```

```vue-html

```

Content inside the `` outlet will be treated as "fallback" content: it will be displayed if the parent did not pass down any slot content:

```vue-html
Fallback content
```

Currently we are not passing any slot content to ``, so you should see the fallback content. Let's provide some slot content to the child while making use of the parent's `msg` state.
