# Event Handling 

  

  

## Listening to Events 

We can use the `v-on` directive, which we typically shorten to the `@` symbol, to listen to DOM events and run some JavaScript when they're triggered. The usage would be `v-on:click="handler"` or with the shortcut, `@click="handler"`.

The handler value can be one of the following:

1. **Inline handlers:** Inline JavaScript to be executed when the event is triggered (similar to the native `onclick` attribute).

2. **Method handlers:** A property name or path that points to a method defined on the component.

## Inline Handlers 

Inline handlers are typically used in simple cases, for example:

```js
const count = ref(0)
```

```js
data() {
  return 
}
```

```vue-html
Add 1
Count is: {}
```

[Try it in the Playground](https://play.vuejs.org/#eNo9jssKgzAURH/lko0tgrbbEqX+Q5fZaLxiqHmQ3LgJ+fdqFZcD58xMYp1z1RqRvRgP0itHEJCia4VR2llPkMDjBBkmbzUUG1oII4y0JhBIGw2hh2Znbo+7MLw+WjZ/C4TaLT3hnogPkcgaeMtFyW8j2GmXpWBtN47w5PWBHLhrPzPCKfWDXRHmPsCAaOBfgSOkdH3IGUhpDBWv9/e8vsZZ/gFFhFJN)

[Try it in the Playground](https://play.vuejs.org/#eNo9jcEKgzAQRH9lyKlF0PYqqdR/6DGXaLYo1RjiRgrivzepIizLzu7sm1XUzuVLIFEKObe+d1wpS183eYahtw4DY1UWMJr15ZpmxYAnDt7uF0BxOwXL5Evc0kbxlmyxxZLFyY2CaXSDZkqKZROYJ4tnO/Tt56HEgckyJaraGNxlsVt2u6teHeF40s20EDo9oyGy+CPIYF1xULBt4H6kOZeFiwBZnOFi+wH0B1hk)

## Method Handlers 

The logic for many event handlers will be more complex though, and likely isn't feasible with inline handlers. That's why `v-on` can also accept the name or path of a component method you'd like to call.

For example:

```js
const name = ref('Vue.js')

function greet(event) {
  alert(`Hello $!`)
  // `event` is the native DOM event
  if (event) 
}
```

```js
data() {
  return 
},
methods: {
  greet(event) {
    // `this` inside methods points to the current active instance
    alert(`Hello $!`)
    // `event` is the native DOM event
    if (event) 
  }
}
```

```vue-html

Greet
```

[Try it in the Playground](https://play.vuejs.org/#eNpVj0FLxDAQhf/KMwjtXtq7dBcFQS/qzVMOrWFao2kSkkkvpf/dJIuCEBgm771vZnbx4H23JRJ3YogqaM+IxMlfpNWrd4GxI9CMA3NwK5psbaSVVjkbGXZaCediaJv3RN1XbE5FnZNVrJ3FEoi4pY0sn7BLC0yGArfjMxnjcLsXQrdNJtFxM+Ys0PcYa2CEjuBPylNYb4THtxdUobj0jH/YX3D963gKC5WyvGZ+xR7S5jf01yPzeblhWr2ZmErHw0dizivfK6PV91mKursUl6dSh/4qZ+vQ/+XE8QODonDi)

[Try it in the Playground](https://play.vuejs.org/#eNplUE1LxDAQ/StjEbYL0t5LXRQEvag3Tz00prNtNE1CMilC6X83SUkRhJDJfLz3Jm8tHo2pFo9FU7SOW2Ho0in8MdoSDHhlXhKsnQIYGLHyvL8BLJK3KmcAis3YwOnDY/XlTnt1i2G7i/eMNOnBNRkwWkQqcUFFByVAXUNPk3A9COXEgBkGRgtFDkgDTQjcWxuAwDiJBeMsMcUxszCJlsr+BaXUcLtGwiqut930579KST1IBd5Aqlgie3p/hdTIk+IK//bMGqleEbMjxjC+BZVDIv0+m9CpcNr6MDgkhLORjDBm1H56Iq3ggUvBv++7IhnUFZfnGNt6b4fRtj5wxfYL9p+Sjw==)

A method handler automatically receives the native DOM Event object that triggers it - in the example above, we are able to access the element dispatching the event via `event.target`.

See also: [Typing Event Handlers](/guide/typescript/composition-api#typing-event-handlers) 

See also: [Typing Event Handlers](/guide/typescript/options-api#typing-event-handlers) 

### Method vs. Inline Detection 

The template compiler detects method handlers by checking whether the `v-on` value string is a valid JavaScript identifier or property access path. For example, `foo`, `foo.bar` and `foo['bar']` are treated as method handlers, while `foo()` and `count++` are treated as inline handlers.

## Calling Methods in Inline Handlers 

Instead of binding directly to a method name, we can also call methods in an inline handler. This allows us to pass the method custom arguments instead of the native event:

```js
function say(message) 
```

```js
methods: {
  say(message) 
}
```

```vue-html
Say hello
Say bye
```

[Try it in the Playground](https://play.vuejs.org/#eNp9jTEOwjAMRa8SeSld6I5CBWdg9ZJGBiJSN2ocpKjq3UmpFDGx+Vn//b/ANYTjOxGcQEc7uyAqkqTQI98TW3ETq2jyYaQYzYNatSArZTzNUn/IK7Ludr2IBYTG4I3QRqKHJFJ6LtY7+zojbIXNk7yfmhahv5msvqS7PfnHGjJVp9w/hu7qKKwfEd1NSg==)

[Try it in the Playground](https://play.vuejs.org/#eNptjUEKwjAQRa8yZFO7sfsSi57B7WzGdjTBtA3NVC2ldzehEFwIw8D7vM9f1cX742tmVSsd2sl6aXDgjx8ngY7vNDuBFQeAnsWMXagToQAEWg49h0APLncDAIUcT5LzlKJsqRBfPF3ljQjCvXcknEj0bRYZBzi3zrbPE6o0UBhblKiaKy1grK52J/oA//23IcmNBD8dXeVBtX0BF0pXsg==)

## Accessing Event Argument in Inline Handlers 

Sometimes we also need to access the original DOM event in an inline handler. You can pass it into a method using the special `$event` variable, or use an inline arrow function:

```vue-html

  Submit

 warn('Form cannot be submitted yet.', event)">
  Submit

```

```js
function warn(message, event) {
  // now we have access to the native event
  if (event) 
  alert(message)
}
```

```js
methods: {
  warn(message, event) {
    // now we have access to the native event
    if (event) 
    alert(message)
  }
}
```

## Event Modifiers 

It is a very common need to call `event.preventDefault()` or `event.stopPropagation()` inside event handlers. Although we can do this easily inside methods, it would be better if the methods can be purely about data logic rather than having to deal with DOM event details.

To address this problem, Vue provides **event modifiers** for `v-on`. Recall that modifiers are directive postfixes denoted by a dot.

- `.stop`
- `.prevent`
- `.self`
- `.capture`
- `.once`
- `.passive`

```vue-html

...
```

::: tip
Order matters when using modifiers because the relevant code is generated in the same order. Therefore using `@click.prevent.self` will prevent **click's default action on the element itself and its children**, while `@click.self.prevent` will only prevent click's default action on the element itself.
:::

The `.capture`, `.once`, and `.passive` modifiers mirror the [options of the native `addEventListener` method](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#options):

```vue-html

...

...
```

The `.passive` modifier is typically used with touch event listeners for [improving performance on mobile devices](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#improving_scroll_performance_using_passive_listeners).

::: tip
Do not use `.passive` and `.prevent` together, because `.passive` already indicates to the browser that you _do not_ intend to prevent the event's default behavior, and you will likely see a warning from the browser if you do so.
:::

## Key Modifiers 

When listening for keyboard events, we often need to check for specific keys. Vue allows adding key modifiers for `v-on` or `@` when listening for key events:

```vue-html

```

You can directly use any valid key names exposed via [`KeyboardEvent.key`](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values) as modifiers by converting them to kebab-case.

```vue-html

```

In the above example, the handler will only be called if `$event.key` is equal to `'PageDown'`.

### Key Aliases 

Vue provides aliases for the most commonly used keys:

- `.enter`
- `.tab`
- `.delete` (captures both "Delete" and "Backspace" keys)
- `.esc`
- `.space`
- `.up`
- `.down`
- `.left`
- `.right`

### System Modifier Keys 

You can use the following modifiers to trigger mouse or keyboard event listeners only when the corresponding modifier key is pressed:

- `.ctrl`
- `.alt`
- `.shift`
- `.meta`

::: tip Note
On Macintosh keyboards, meta is the command key (⌘). On Windows keyboards, meta is the Windows key (⊞). On Sun Microsystems keyboards, meta is marked as a solid diamond (◆). On certain keyboards, specifically MIT and Lisp machine keyboards and successors, such as the Knight keyboard, space-cadet keyboard, meta is labeled “META”. On Symbolics keyboards, meta is labeled “META” or “Meta”.
:::

For example:

```vue-html

Do something
```

::: tip
Note that modifier keys are different from regular keys and when used with `keyup` events, they have to be pressed when the event is emitted. In other words, `keyup.ctrl` will only trigger if you release a key while holding down `ctrl`. It won't trigger if you release the `ctrl` key alone.
:::

### `.exact` Modifier 

The `.exact` modifier allows control of the exact combination of system modifiers needed to trigger an event.

```vue-html

A

A

A
```

## Mouse Button Modifiers 

- `.left`
- `.right`
- `.middle`

These modifiers restrict the handler to events triggered by a specific mouse button.

Note, however, that `.left`, `.right`, and `.middle` modifier names are based on the typical right-handed mouse layout, but in fact represent "main", "secondary", and "auxiliary" pointing device event triggers, respectively, and not the actual physical buttons. So that for a left-handed mouse layout the "main" button might physically be the right one but would trigger the `.left` modifier handler. Or a trackpad might trigger the `.left` handler with a one-finger tap, the `.right` handler with a two-finger tap, and the `.middle` handler with a three-finger tap. Similarly, other devices and event sources generating "mouse" events might have trigger modes that are not related to "left" and "right" whatsoever.
