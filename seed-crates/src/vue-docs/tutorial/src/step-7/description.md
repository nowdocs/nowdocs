# List Rendering 

We can use the `v-for` directive to render a list of elements based on a source array:

```vue-html

  
    {}
  

```

Here `todo` is a local variable representing the array element currently being iterated on. It's only accessible on or inside the `v-for` element, similar to a function scope.

Notice how we are also giving each todo object a unique `id`, and binding it as the special `key` attribute for each ``. The `key` allows Vue to accurately move each `` to match the position of its corresponding object in the array.

There are two ways to update the list:

1. Call [mutating methods](https://stackoverflow.com/questions/9009879/which-javascript-array-functions-are-mutating) on the source array:

   

   ```js
   todos.value.push(newTodo)
   ```

     
     

   ```js
   this.todos.push(newTodo)
   ```

   

2. Replace the array with a new one:

   

   ```js
   todos.value = todos.value.filter(/* ... */)
   ```

     
     

   ```js
   this.todos = this.todos.filter(/* ... */)
   ```

   

Here we have a simple todo list - try to implement the logic for `addTodo()` and `removeTodo()` methods to make it work!

More details on `v-for`: Guide - List Rendering
