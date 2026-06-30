# Form Bindings 

Using `v-bind` and `v-on` together, we can create two-way bindings on form input elements:

```vue-html

```

```js
methods: {
  onInput(e) 
}
```

```js
function onInput(e) 
```

Try typing in the input box - you should see the text in `` updating as you type.

To simplify two-way bindings, Vue provides a directive, `v-model`, which is essentially syntactic sugar for the above:

```vue-html

```

`v-model` automatically syncs the ``'s value with the bound state, so we no longer need to use an event handler for that.

`v-model` works not only on text inputs, but also on other input types such as checkboxes, radio buttons, and select dropdowns. We cover more details in Guide - Form Bindings.

Now, try to refactor the code to use `v-model` instead.
