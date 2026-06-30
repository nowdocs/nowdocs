# Priority D Rules: Use with Caution 

Some features of Vue exist to accommodate rare edge cases or smoother migrations from a legacy code base. When overused however, they can make your code more difficult to maintain or even become a source of bugs. These rules shine a light on potentially risky features, describing when and why they should be avoided.

## Element selectors with `scoped` 

**Element selectors should be avoided with `scoped`.**

Prefer class selectors over element selectors in `scoped` styles, because large numbers of element selectors are slow.

::: details Detailed Explanation
To scope styles, Vue adds a unique attribute to component elements, such as `data-v-f3f3eg9`. Then selectors are modified so that only matching elements with this attribute are selected (e.g. `button[data-v-f3f3eg9]`).

The problem is that large numbers of element-attribute selectors (e.g. `button[data-v-f3f3eg9]`) will be considerably slower than class-attribute selectors (e.g. `.btn-close[data-v-f3f3eg9]`), so class selectors should be preferred whenever possible.
:::

Bad

```vue-html

  ×

button 

```

Good

```vue-html

  ×

.btn-close 

```

## Implicit parent-child communication 

**Props and events should be preferred for parent-child component communication, instead of `this.$parent` or mutating props.**

An ideal Vue application is props down, events up. Sticking to this convention makes your components much easier to understand. However, there are edge cases where prop mutation or `this.$parent` can simplify two components that are already deeply coupled.

The problem is, there are also many _simple_ cases where these patterns may offer convenience. Beware: do not be seduced into trading simplicity (being able to understand the flow of your state) for short-term convenience (writing less code).

Bad

```js
app.component('TodoItem', {
  props: {
    todo: 
  },

  template: ''
})
```

```js
app.component('TodoItem', {
  props: {
    todo: 
  },

  methods: {
    removeTodo() 
  },

  template: `
    
      {}
      
        ×
      
    
  `
})
```

Good

```js
app.component('TodoItem', {
  props: {
    todo: 
  },

  emits: ['input'],

  template: `
    
  `
})
```

```js
app.component('TodoItem', {
  props: {
    todo: 
  },

  emits: ['delete'],

  template: `
    
      {}
      
        ×
      
    
  `
})
```

Bad

```vue

defineProps({
  todo: 
})

  

```

```vue

const props = defineProps({
  todo: 
})

function renameTodo() 

  
    {}
    rename
  

```

Good

```vue

defineProps({
  todo: 
})

const emit = defineEmits(['input'])

  

```

```vue

const props = defineProps({
  todo: 
})

const emit = defineEmits(['update:todo'])

function renameTodo() {
  // Emit a new object — the parent owns the update.
  emit('update:todo', )
}

  
    {}
    rename
  

```

