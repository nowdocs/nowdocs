---
title: 'Removing Effect Dependencies'
---

## Dependencies should match the code 

When you write an Effect, you first specify how to [start and stop](/learn/lifecycle-of-reactive-effects#the-lifecycle-of-an-effect) whatever you want your Effect to be doing:

```js 
const serverUrl = 'https://localhost:1234';

function ChatRoom() {
  useEffect(() => 
```

Then, if you leave the Effect dependencies empty (`[]`), the linter will suggest the correct dependencies:

Fill them in according to what the linter says:

```js 
function ChatRoom() {
  useEffect(() => , [roomId]); // ✅ All dependencies declared
  // ...
}
```

[Effects "react" to reactive values.](/learn/lifecycle-of-reactive-effects#effects-react-to-reactive-values) Since `roomId` is a reactive value (it can change due to a re-render), the linter verifies that you've specified it as a dependency. If `roomId` receives a different value, React will re-synchronize your Effect. This ensures that the chat stays connected to the selected room and "reacts" to the dropdown:

### To remove a dependency, prove that it's not a dependency 

Notice that you can't "choose" the dependencies of your Effect. Every  used by your Effect's code must be declared in your dependency list. The dependency list is determined by the surrounding code:

```js [[2, 3, "roomId"], [2, 5, "roomId"], [2, 8, "roomId"]]
const serverUrl = 'https://localhost:1234';

function ChatRoom() { // This is a reactive value
  useEffect(() => , [roomId]); // ✅ So you must specify that reactive value as a dependency of your Effect
  // ...
}
```

[Reactive values](/learn/lifecycle-of-reactive-effects#all-variables-declared-in-the-component-body-are-reactive) include props and all variables and functions declared directly inside of your component. Since `roomId` is a reactive value, you can't remove it from the dependency list. The linter wouldn't allow it:

```js 
const serverUrl = 'https://localhost:1234';

function ChatRoom() {
  useEffect(() => , []); // 🔴 React Hook useEffect has a missing dependency: 'roomId'
  // ...
}
```

And the linter would be right! Since `roomId` may change over time, this would introduce a bug in your code.

**To remove a dependency, "prove" to the linter that it *doesn't need* to be a dependency.** For example, you can move `roomId` out of your component to prove that it's not reactive and won't change on re-renders:

```js 
const serverUrl = 'https://localhost:1234';
const roomId = 'music'; // Not a reactive value anymore

function ChatRoom() {
  useEffect(() => , []); // ✅ All dependencies declared
  // ...
}
```

Now that `roomId` is not a reactive value (and can't change on a re-render), it doesn't need to be a dependency:

This is why you could now specify an [empty (`[]`) dependency list.](/learn/lifecycle-of-reactive-effects#what-an-effect-with-empty-dependencies-means) Your Effect *really doesn't* depend on any reactive value anymore, so it *really doesn't* need to re-run when any of the component's props or state change.

### To change the dependencies, change the code 

You might have noticed a pattern in your workflow:

1. First, you **change the code** of your Effect or how your reactive values are declared.
2. Then, you follow the linter and adjust the dependencies to **match the code you have changed.**
3. If you're not happy with the list of dependencies, you **go back to the first step** (and change the code again).

The last part is important. **If you want to change the dependencies, change the surrounding code first.** You can think of the dependency list as [a list of all the reactive values used by your Effect's code.](/learn/lifecycle-of-reactive-effects#react-verifies-that-you-specified-every-reactive-value-as-a-dependency) You don't *choose* what to put on that list. The list *describes* your code. To change the dependency list, change the code.

This might feel like solving an equation. You might start with a goal (for example, to remove a dependency), and you need to "find" the code matching that goal. Not everyone finds solving equations fun, and the same thing could be said about writing Effects! Luckily, there is a list of common recipes that you can try below.

Let's say that you wanted to run the Effect "only on mount". You've read that [empty (`[]`) dependencies](/learn/lifecycle-of-reactive-effects#what-an-effect-with-empty-dependencies-means) do that, so you've decided to ignore the linter, and forcefully specified `[]` as the dependencies.

This counter was supposed to increment every second by the amount configurable with the two buttons. However, since you "lied" to React that this Effect doesn't depend on anything, React forever keeps using the `onTick` function from the initial render. [During that render,](/learn/state-as-a-snapshot#rendering-takes-a-snapshot-in-time) `count` was `0` and `increment` was `1`. This is why `onTick` from that render always calls `setCount(0 + 1)` every second, and you always see `1`. Bugs like this are harder to fix when they're spread across multiple components.

There's always a better solution than ignoring the linter! To fix this code, you need to add `onTick` to the dependency list. (To ensure the interval is only setup once, [make `onTick` an Effect Event.](/learn/separating-events-from-effects#reading-latest-props-and-state-with-effect-events))

**We recommend treating the dependency lint error as a compilation error. If you don't suppress it, you will never see bugs like this.** The rest of this page documents the alternatives for this and other cases.

## Removing unnecessary dependencies 

Every time you adjust the Effect's dependencies to reflect the code, look at the dependency list. Does it make sense for the Effect to re-run when any of these dependencies change? Sometimes, the answer is "no":

* You might want to re-execute *different parts* of your Effect under different conditions.
* You might want to only read the *latest value* of some dependency instead of "reacting" to its changes.
* A dependency may change too often *unintentionally* because it's an object or a function.

To find the right solution, you'll need to answer a few questions about your Effect. Let's walk through them.

### Should this code move to an event handler? 

The first thing you should think about is whether this code should be an Effect at all.

Imagine a form. On submit, you set the `submitted` state variable to `true`. You need to send a POST request and show a notification. You've put this logic inside an Effect that "reacts" to `submitted` being `true`:

```js 
function Form() {
  const [submitted, setSubmitted] = useState(false);

  useEffect(() => {
    if (submitted) 
  }, [submitted]);

  function handleSubmit() 

  // ...
}
```

Later, you want to style the notification message according to the current theme, so you read the current theme. Since `theme` is declared in the component body, it is a reactive value, so you add it as a dependency:

```js 
function Form() {
  const [submitted, setSubmitted] = useState(false);
  const theme = useContext(ThemeContext);

  useEffect(() => {
    if (submitted) 
  }, [submitted, theme]); // ✅ All dependencies declared

  function handleSubmit() 

  // ...
}
```

By doing this, you've introduced a bug. Imagine you submit the form first and then switch between Dark and Light themes. The `theme` will change, the Effect will re-run, and so it will display the same notification again!

**The problem here is that this shouldn't be an Effect in the first place.** You want to send this POST request and show the notification in response to *submitting the form,* which is a particular interaction. To run some code in response to particular interaction, put that logic directly into the corresponding event handler:

```js 
function Form() {
  const theme = useContext(ThemeContext);

  function handleSubmit() 

  // ...
}
```

Now that the code is in an event handler, it's not reactive--so it will only run when the user submits the form. Read more about [choosing between event handlers and Effects](/learn/separating-events-from-effects#reactive-values-and-reactive-logic) and [how to delete unnecessary Effects.](/learn/you-might-not-need-an-effect)

### Is your Effect doing several unrelated things? 

The next question you should ask yourself is whether your Effect is doing several unrelated things.

Imagine you're creating a shipping form where the user needs to choose their city and area. You fetch the list of `cities` from the server according to the selected `country` to show them in a dropdown:

```js
function ShippingForm() {
  const [cities, setCities] = useState(null);
  const [city, setCity] = useState(null);

  useEffect(() => {
    let ignore = false;
    fetch(`/api/cities?country=$`)
      .then(response => response.json())
      .then(json => {
        if (!ignore) 
      });
    return () => ;
  }, [country]); // ✅ All dependencies declared

  // ...
```

This is a good example of [fetching data in an Effect.](/learn/you-might-not-need-an-effect#fetching-data) You are synchronizing the `cities` state with the network according to the `country` prop. You can't do this in an event handler because you need to fetch as soon as `ShippingForm` is displayed and whenever the `country` changes (no matter which interaction causes it).

Now let's say you're adding a second select box for city areas, which should fetch the `areas` for the currently selected `city`. You might start by adding a second `fetch` call for the list of areas inside the same Effect:

```js 
function ShippingForm() {
  const [cities, setCities] = useState(null);
  const [city, setCity] = useState(null);
  const [areas, setAreas] = useState(null);

  useEffect(() => {
    let ignore = false;
    fetch(`/api/cities?country=$`)
      .then(response => response.json())
      .then(json => {
        if (!ignore) 
      });
    // 🔴 Avoid: A single Effect synchronizes two independent processes
    if (city) {
      fetch(`/api/areas?city=$`)
        .then(response => response.json())
        .then(json => {
          if (!ignore) 
        });
    }
    return () => ;
  }, [country, city]); // ✅ All dependencies declared

  // ...
```

However, since the Effect now uses the `city` state variable, you've had to add `city` to the list of dependencies. That, in turn, introduced a problem: when the user selects a different city, the Effect will re-run and call `fetchCities(country)`. As a result, you will be unnecessarily refetching the list of cities many times.

**The problem with this code is that you're synchronizing two different unrelated things:**

1. You want to synchronize the `cities` state to the network based on the `country` prop.
1. You want to synchronize the `areas` state to the network based on the `city` state.

Split the logic into two Effects, each of which reacts to the prop that it needs to synchronize with:

```js 
function ShippingForm() {
  const [cities, setCities] = useState(null);
  useEffect(() => {
    let ignore = false;
    fetch(`/api/cities?country=$`)
      .then(response => response.json())
      .then(json => {
        if (!ignore) 
      });
    return () => ;
  }, [country]); // ✅ All dependencies declared

  const [city, setCity] = useState(null);
  const [areas, setAreas] = useState(null);
  useEffect(() => {
    if (city) {
      let ignore = false;
      fetch(`/api/areas?city=$`)
        .then(response => response.json())
        .then(json => {
          if (!ignore) 
        });
      return () => ;
    }
  }, [city]); // ✅ All dependencies declared

  // ...
```

Now the first Effect only re-runs if the `country` changes, while the second Effect re-runs when the `city` changes. You've separated them by purpose: two different things are synchronized by two separate Effects. Two separate Effects have two separate dependency lists, so they won't trigger each other unintentionally.

The final code is longer than the original, but splitting these Effects is still correct. [Each Effect should represent an independent synchronization process.](/learn/lifecycle-of-reactive-effects#each-effect-represents-a-separate-synchronization-process) In this example, deleting one Effect doesn't break the other Effect's logic. This means they *synchronize different things,* and it's good to split them up. If you're concerned about duplication, you can improve this code by [extracting repetitive logic into a custom Hook.](/learn/reusing-logic-with-custom-hooks#when-to-use-custom-hooks)

### Are you reading some state to calculate the next state? 

This Effect updates the `messages` state variable with a newly created array every time a new message arrives:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);
  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => );
    // ...
```

It uses the `messages` variable to [create a new array](/learn/updating-arrays-in-state) starting with all the existing messages and adds the new message at the end. However, since `messages` is a reactive value read by an Effect, it must be a dependency:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);
  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => );
    return () => connection.disconnect();
  }, [roomId, messages]); // ✅ All dependencies declared
  // ...
```

And making `messages` a dependency introduces a problem.

Every time you receive a message, `setMessages()` causes the component to re-render with a new `messages` array that includes the received message. However, since this Effect now depends on `messages`, this will *also* re-synchronize the Effect. So every new message will make the chat re-connect. The user would not like that!

To fix the issue, don't read `messages` inside the Effect. Instead, pass an [updater function](/reference/react/useState#updating-state-based-on-the-previous-state) to `setMessages`:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);
  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => );
    return () => connection.disconnect();
  }, [roomId]); // ✅ All dependencies declared
  // ...
```

**Notice how your Effect does not read the `messages` variable at all now.** You only need to pass an updater function like `msgs => [...msgs, receivedMessage]`. React [puts your updater function in a queue](/learn/queueing-a-series-of-state-updates) and will provide the `msgs` argument to it during the next render. This is why the Effect itself doesn't need to depend on `messages` anymore. As a result of this fix, receiving a chat message will no longer make the chat re-connect.

### Do you want to read a value without "reacting" to its changes? 

Suppose that you want to play a sound when the user receives a new message unless `isMuted` is `true`:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);
  const [isMuted, setIsMuted] = useState(false);

  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => {
      setMessages(msgs => [...msgs, receivedMessage]);
      if (!isMuted) 
    });
    // ...
```

Since your Effect now uses `isMuted` in its code, you have to add it to the dependencies:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);
  const [isMuted, setIsMuted] = useState(false);

  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => {
      setMessages(msgs => [...msgs, receivedMessage]);
      if (!isMuted) 
    });
    return () => connection.disconnect();
  }, [roomId, isMuted]); // ✅ All dependencies declared
  // ...
```

The problem is that every time `isMuted` changes (for example, when the user presses the "Muted" toggle), the Effect will re-synchronize, and reconnect to the chat. This is not the desired user experience! (In this example, even disabling the linter would not work--if you do that, `isMuted` would get "stuck" with its old value.)

To solve this problem, you need to extract the logic that shouldn't be reactive out of the Effect. You don't want this Effect to "react" to the changes in `isMuted`. [Move this non-reactive piece of logic into an Effect Event:](/learn/separating-events-from-effects#declaring-an-effect-event)

```js 
import  from 'react';

function ChatRoom() {
  const [messages, setMessages] = useState([]);
  const [isMuted, setIsMuted] = useState(false);

  const onMessage = useEffectEvent(receivedMessage => {
    setMessages(msgs => [...msgs, receivedMessage]);
    if (!isMuted) 
  });

  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => );
    return () => connection.disconnect();
  }, [roomId]); // ✅ All dependencies declared
  // ...
```

Effect Events let you split an Effect into reactive parts (which should "react" to reactive values like `roomId` and their changes) and non-reactive parts (which only read their latest values, like `onMessage` reads `isMuted`). **Now that you read `isMuted` inside an Effect Event, it doesn't need to be a dependency of your Effect.** As a result, the chat won't re-connect when you toggle the "Muted" setting on and off, solving the original issue!

#### Wrapping an event handler from the props 

You might run into a similar problem when your component receives an event handler as a prop:

```js 
function ChatRoom() {
  const [messages, setMessages] = useState([]);

  useEffect(() => {
    const connection = createConnection();
    connection.connect();
    connection.on('message', (receivedMessage) => );
    return () => connection.disconnect();
  }, [roomId, onReceiveMessage]); // ✅ All dependencies declared
  // ...
```

Suppose that the parent component passes a *different* `onReceiveMessage` function on every render:

```js 

In the sandbox above, the input only updates the `message` state variable. From the user's perspective, this should not affect the chat connection. However, every time you update the `message`, your component re-renders. When your component re-renders, the code inside of it runs again from scratch.

A new `options` object is created from scratch on every re-render of the `ChatRoom` component. React sees that the `options` object is a *different object* from the `options` object created during the last render. This is why it re-synchronizes your Effect (which depends on `options`), and the chat re-connects as you type.

**This problem only affects objects and functions. In JavaScript, each newly created object and function is considered distinct from all the others. It doesn't matter that the contents inside of them may be the same!**

```js 
// During the first render
const options1 = ;

// During the next render
const options2 = ;

// These are two different objects!
console.log(Object.is(options1, options2)); // false
```

**Object and function dependencies can make your Effect re-synchronize more often than you need.**

This is why, whenever possible, you should try to avoid objects and functions as your Effect's dependencies. Instead, try moving them outside the component, inside the Effect, or extracting primitive values out of them.

#### Move static objects and functions outside your component 

If the object does not depend on any props and state, you can move that object outside your component:

```js 
const options = ;

function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => , []); // ✅ All dependencies declared
  // ...
```

This way, you *prove* to the linter that it's not reactive. It can't change as a result of a re-render, so it doesn't need to be a dependency. Now re-rendering `ChatRoom` won't cause your Effect to re-synchronize.

This works for functions too:

```js 
function createOptions() {
  return ;
}

function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => , []); // ✅ All dependencies declared
  // ...
```

Since `createOptions` is declared outside your component, it's not a reactive value. This is why it doesn't need to be specified in your Effect's dependencies, and why it won't ever cause your Effect to re-synchronize.

#### Move dynamic objects and functions inside your Effect 

If your object depends on some reactive value that may change as a result of a re-render, like a `roomId` prop, you can't pull it *outside* your component. You can, however, move its creation *inside* of your Effect's code:

```js 
const serverUrl = 'https://localhost:1234';

function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => {
    const options = ;
    const connection = createConnection(options);
    connection.connect();
    return () => connection.disconnect();
  }, [roomId]); // ✅ All dependencies declared
  // ...
```

Now that `options` is declared inside of your Effect, it is no longer a dependency of your Effect. Instead, the only reactive value used by your Effect is `roomId`. Since `roomId` is not an object or function, you can be sure that it won't be *unintentionally* different. In JavaScript, numbers and strings are compared by their content:

```js 
// During the first render
const roomId1 = 'music';

// During the next render
const roomId2 = 'music';

// These two strings are the same!
console.log(Object.is(roomId1, roomId2)); // true
```

Thanks to this fix, the chat no longer re-connects if you edit the input:

However, it *does* re-connect when you change the `roomId` dropdown, as you would expect.

This works for functions, too:

```js 
const serverUrl = 'https://localhost:1234';

function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => {
    function createOptions() {
      return ;
    }

    const options = createOptions();
    const connection = createConnection(options);
    connection.connect();
    return () => connection.disconnect();
  }, [roomId]); // ✅ All dependencies declared
  // ...
```

You can write your own functions to group pieces of logic inside your Effect. As long as you also declare them *inside* your Effect, they're not reactive values, and so they don't need to be dependencies of your Effect.

#### Read primitive values from objects 

Sometimes, you may receive an object from props:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => , [options]); // ✅ All dependencies declared
  // ...
```

The risk here is that the parent component will create the object during rendering:

```js 

Instead of reading `count` inside the Effect, you pass a `c => c + 1` instruction ("increment this number!") to React. React will apply it on the next render. And since you don't need to read the value of `count` inside your Effect anymore, you can keep your Effect's dependencies empty (`[]`). This prevents your Effect from re-creating the interval on every tick.

#### Fix a retriggering animation 

In this example, when you press "Show", a welcome message fades in. The animation takes a second. When you press "Remove", the welcome message immediately disappears. The logic for the fade-in animation is implemented in the `animation.js` file as plain JavaScript [animation loop.](https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame) You don't need to change that logic. You can treat it as a third-party library. Your Effect creates an instance of `FadeInAnimation` for the DOM node, and then calls `start(duration)` or `stop()` to control the animation. The `duration` is controlled by a slider. Adjust the slider and see how the animation changes.

This code already works, but there is something you want to change. Currently, when you move the slider that controls the `duration` state variable, it retriggers the animation. Change the behavior so that the Effect does not "react" to the `duration` variable. When you press "Show", the Effect should use the current `duration` on the slider. However, moving the slider itself should not by itself retrigger the animation.

Effect Events like `onAppear` are not reactive, so you can read `duration` inside without retriggering the animation.

#### Fix a reconnecting chat 

In this example, every time you press "Toggle theme", the chat re-connects. Why does this happen? Fix the mistake so that the chat re-connects only when you edit the Server URL or choose a different chat room.

Treat `chat.js` as an external third-party library: you can consult it to check its API, but don't edit it.

It would be even better to replace the object `options` prop with the more specific `roomId` and `serverUrl` props:

Sticking to primitive props where possible makes it easier to optimize your components later.

#### Fix a reconnecting chat, again 

This example connects to the chat either with or without encryption. Toggle the checkbox and notice the different messages in the console when the encryption is on and off. Try changing the room. Then, try toggling the theme. When you're connected to a chat room, you will receive new messages every few seconds. Verify that their color matches the theme you've picked.

In this example, the chat re-connects every time you try to change the theme. Fix this. After the fix, changing the theme should not re-connect the chat, but toggling encryption settings or changing the room should re-connect.

Don't change any code in `chat.js`. Other than that, you can change any code as long as it results in the same behavior. For example, you may find it helpful to change which props are being passed down.

