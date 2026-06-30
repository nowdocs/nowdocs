# Watchers 

Sometimes we may need to perform "side effects" reactively - for example, logging a number to the console when it changes. We can achieve this with watchers:

```js
import  from 'vue'

const count = ref(0)

watch(count, (newCount) => {
  // yes, console.log() is a side effect
  console.log(`new count is: $`)
})
```

`watch()` can directly watch a ref, and the callback gets fired whenever `count`'s value changes. `watch()` can also watch other types of data sources - more details are covered in Guide - Watchers.

```js
export default {
  data() {
    return 
  },
  watch: {
    count(newCount) {
      // yes, console.log() is a side effect
      console.log(`new count is: $`)
    }
  }
}
```

Here, we are using the `watch` option to watch changes to the `count` property. The watch callback is called when `count` changes, and receives the new value as the argument. More details are covered in Guide - Watchers.

A more practical example than logging to the console would be fetching new data when an ID changes. The code we have is fetching todos data from a mock API on component mount. There is also a button that increments the todo ID that should be fetched. Try to implement a watcher that fetches a new todo when the button is clicked.
