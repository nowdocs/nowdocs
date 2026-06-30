
import ElasticHeader from './demos/ElasticHeader.vue'
import DisabledButton from './demos/DisabledButton.vue'
import Colors from './demos/Colors.vue'
import AnimateWatcher from './demos/AnimateWatcher.vue'

# Animation Techniques 

Vue provides the [``](/guide/built-ins/transition) and [``](/guide/built-ins/transition-group) components for handling enter / leave and list transitions. However, there are many other ways of using animations on the web, even in a Vue application. Here we will discuss a few additional techniques.

## Class-based Animations 

For elements that are not entering / leaving the DOM, we can trigger animations by dynamically adding a CSS class:

```js
const disabled = ref(false)

function warnDisabled() {
  disabled.value = true
  setTimeout(() => , 1500)
}
```

```js
export default {
  data() {
    return 
  },
  methods: {
    warnDisabled() {
      this.disabled = true
      setTimeout(() => , 1500)
    }
  }
}
```

```vue-html

  Click me
  This feature is disabled!

```

```css
.shake 

@keyframes shake {
  10%,
  90% 

  20%,
  80% 

  30%,
  50%,
  70% 

  40%,
  60% 
}
```

## State-driven Animations 

Some transition effects can be applied by interpolating values, for instance by binding a style to an element while an interaction occurs. Take this example for instance:

```js
const x = ref(0)

function onMousemove(e) 
```

```js
export default {
  data() {
    return 
  },
  methods: {
    onMousemove(e) 
  }
}
```

```vue-html

  Move your mouse across this div...
  x: {}

```

```css
.movearea 
```

In addition to color, you can also use style bindings to animate transform, width, or height. You can even animate SVG paths using spring physics - after all, they are all attribute data bindings:

## Animating with Watchers 

With some creativity, we can use watchers to animate anything based on some numerical state. For example, we can animate the number itself:

```js
import  from 'vue'
import gsap from 'gsap'

const number = ref(0)
const tweened = reactive()

// Note: For inputs greater than Number.MAX_SAFE_INTEGER (9007199254740991),
// the result may be inaccurate due to limitations in JavaScript number precision.
watch(number, (n) => {
  gsap.to(tweened, )
})
```

```vue-html
Type a number: 
{}
```

```js
import gsap from 'gsap'

export default {
  data() {
    return 
  },
  // Note: For inputs greater than Number.MAX_SAFE_INTEGER (9007199254740991),
  // the result may be inaccurate due to limitations in JavaScript number precision.
  watch: {
    number(n) {
      gsap.to(this, )
    }
  }
}
```

```vue-html
Type a number: 
{}
```

[Try it in the Playground](https://play.vuejs.org/#eNpNUstygzAM/BWNLyEzBDKd6YWSdHrpsacefSGgJG7xY7BImhL+vTKv9ILllXYlr+jEm3PJpUWRidyXjXIEHql1e2mUdrYh6KDBY8yfoiR1wRiuBZVn6OHYWA0r5q6W2pMv3ISHkBPSlNZ4AtPqAzawC2LRdj3DdEU0WA34qB910sBUnsFWmp6LpRmaRo9UHMLIrGG3h4EBQ/OEbDRpxjx51TYFKWtYKHmOF9WP4Qzs+x22EDoA9NLwmaejC/x+vhBqVxeEfAPIK3WBsi6830lRobZSDDjA580hFIt8roxrCS4bbSuskxFmzhhIAenEy92id1CnzZzfd91szETmZ72rH6zYOej7PA3rYXrKE3GUp//m5KunWx3C5CE6enS0hjZXVKczZXCwdfWyoF79YgZPqBliJ9iGSUTEYlzuRrO9X94a/lUGNTklvBTZvAMpwhYCIMWZyPksTVvjvk9JaXUacq9sSlujFJPnvej/AElH3FQ=)

[Try it in the Playground](https://play.vuejs.org/#eNpNUctugzAQ/JWVLyESj6hSL5Sm6qXHnnr0xYENuAXbwus8Svj3GlxIJEvendHMvgb2bkx6cshyVtiyl4b2XMnO6J6gtsLAsdcdbKZwwxVXeJmpCo/CtQQDVwCVIBFtQwzQI7leLRmAct0B+xx28YLQGVFh5aGAjNM3zvRZUNnkizhII7V6w9xTSjqiRtoYBqhcL0hq5c3S5/hu/blKbzfYwbh9LMWVf0W2zusTws60gnDK6OtqEMTaeSGVcQSnpNMVtmmAXzkLAWeQzarCQNkKaz1zkHWysPthWNryjX/IC1bRbgvjWGTG64rssbQqLF3bKUzvHmH6o1aUnFHWDeVw0G31sqJW/mIOT9h5KEw2m7CYhUsmnV/at9XKX3n24v+E5WxdNmfTbieAs4bI2DzLnDI/dVrqLpu4Nz+/a5GzZYls/AM3dcFx)

