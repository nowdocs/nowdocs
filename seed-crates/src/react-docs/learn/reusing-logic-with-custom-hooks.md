---
title: 'Reusing Logic with Custom Hooks'
---

## Custom Hooks: Sharing logic between components 

Imagine you're developing an app that heavily relies on the network (as most apps do). You want to warn the user if their network connection has accidentally gone off while they were using your app. How would you go about it? It seems like you'll need two things in your component:

1. A piece of state that tracks whether the network is online.
2. An Effect that subscribes to the global [`online`](https://developer.mozilla.org/en-US/docs/Web/API/Window/online_event) and [`offline`](https://developer.mozilla.org/en-US/docs/Web/API/Window/offline_event) events, and updates that state.

This will keep your component [synchronized](/learn/synchronizing-with-effects) with the network status. You might start with something like this:

Try turning your network on and off, and notice how this `StatusBar` updates in response to your actions.

Now imagine you *also* want to use the same logic in a different component. You want to implement a Save button that will become disabled and show "Reconnecting..." instead of "Save" while the network is off.

To start, you can copy and paste the `isOnline` state and the Effect into `SaveButton`:

Verify that, if you turn off the network, the button will change its appearance.

These two components work fine, but the duplication in logic between them is unfortunate. It seems like even though they have different *visual appearance,* you want to reuse the logic between them.

### Extracting your own custom Hook from a component 

Imagine for a moment that, similar to [`useState`](/reference/react/useState) and [`useEffect`](/reference/react/useEffect), there was a built-in `useOnlineStatus` Hook. Then both of these components could be simplified and you could remove the duplication between them:

```js 
function StatusBar() {
  const isOnline = useOnlineStatus();
  return ;
}

function SaveButton() {
  const isOnline = useOnlineStatus();

  function handleSaveClick() 

  return (
    
      
    
  );
}
```

Although there is no such built-in Hook, you can write it yourself. Declare a function called `useOnlineStatus` and move all the duplicated code into it from the components you wrote earlier:

```js 
function useOnlineStatus() {
  const [isOnline, setIsOnline] = useState(true);
  useEffect(() => {
    function handleOnline() 
    function handleOffline() 
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    return () => ;
  }, []);
  return isOnline;
}
```

At the end of the function, return `isOnline`. This lets your components read that value:

Verify that switching the network on and off updates both components.

Now your components don't have as much repetitive logic. **More importantly, the code inside them describes *what they want to do* (use the online status!) rather than *how to do it* (by subscribing to the browser events).**

When you extract logic into custom Hooks, you can hide the gnarly details of how you deal with some external system or a browser API. The code of your components expresses your intent, not the implementation.

### Hook names always start with `use` 

React applications are built from components. Components are built from Hooks, whether built-in or custom. You'll likely often use custom Hooks created by others, but occasionally you might write one yourself!

You must follow these naming conventions:

1. **React component names must start with a capital letter,** like `StatusBar` and `SaveButton`. React components also need to return something that React knows how to display, like a piece of JSX.
2. **Hook names must start with `use` followed by a capital letter,** like [`useState`](/reference/react/useState) (built-in) or `useOnlineStatus` (custom, like earlier on the page). Hooks may return arbitrary values.

This convention guarantees that you can always look at a component and know where its state, Effects, and other React features might "hide". For example, if you see a `getColor()` function call inside your component, you can be sure that it can't possibly contain React state inside because its name doesn't start with `use`. However, a function call like `useOnlineStatus()` will most likely contain calls to other Hooks inside!

### Custom Hooks let you share stateful logic, not state itself 

In the earlier example, when you turned the network on and off, both components updated together. However, it's wrong to think that a single `isOnline` state variable is shared between them. Look at this code:

```js 
function StatusBar() 

function SaveButton() 
```

It works the same way as before you extracted the duplication:

```js 
function StatusBar() {
  const [isOnline, setIsOnline] = useState(true);
  useEffect(() => , []);
  // ...
}

function SaveButton() {
  const [isOnline, setIsOnline] = useState(true);
  useEffect(() => , []);
  // ...
}
```

These are two completely independent state variables and Effects! They happened to have the same value at the same time because you synchronized them with the same external value (whether the network is on).

To better illustrate this, we'll need a different example. Consider this `Form` component:

There's some repetitive logic for each form field:

1. There's a piece of state (`firstName` and `lastName`).
1. There's a change handler (`handleFirstNameChange` and `handleLastNameChange`).
1. There's a piece of JSX that specifies the `value` and `onChange` attributes for that input.

You can extract the repetitive logic into this `useFormInput` custom Hook:

Notice that it only declares *one* state variable called `value`.

However, the `Form` component calls `useFormInput` *two times:*

```js
function Form() {
  const firstNameProps = useFormInput('Mary');
  const lastNameProps = useFormInput('Poppins');
  // ...
```

This is why it works like declaring two separate state variables!

**Custom Hooks let you share *stateful logic* but not *state itself.* Each call to a Hook is completely independent from every other call to the same Hook.** This is why the two sandboxes above are completely equivalent. If you'd like, scroll back up and compare them. The behavior before and after extracting a custom Hook is identical.

When you need to share the state itself between multiple components, [lift it up and pass it down](/learn/sharing-state-between-components) instead.

## Passing reactive values between Hooks 

The code inside your custom Hooks will re-run during every re-render of your component. This is why, like components, custom Hooks [need to be pure.](/learn/keeping-components-pure) Think of custom Hooks' code as part of your component's body!

Because custom Hooks re-render together with your component, they always receive the latest props and state. To see what this means, consider this chat room example. Change the server URL or the chat room:

When you change `serverUrl` or `roomId`, the Effect ["reacts" to your changes](/learn/lifecycle-of-reactive-effects#effects-react-to-reactive-values) and re-synchronizes. You can tell by the console messages that the chat re-connects every time that you change your Effect's dependencies.

Now move the Effect's code into a custom Hook:

```js 
export function useChatRoom() {
  useEffect(() => {
    const options = ;
    const connection = createConnection(options);
    connection.connect();
    connection.on('message', (msg) => );
    return () => connection.disconnect();
  }, [roomId, serverUrl]);
}
```

This lets your `ChatRoom` component call your custom Hook without worrying about how it works inside:

```js 
export default function ChatRoom() {
  const [serverUrl, setServerUrl] = useState('https://localhost:1234');

  useChatRoom();

  return (
    <>
      
        Server URL:
         setServerUrl(e.target.value)} />
      
      Welcome to the  room!
    </>
  );
}
```

This looks much simpler! (But it does the same thing.)

Notice that the logic *still responds* to prop and state changes. Try editing the server URL or the selected room:

Notice how you're taking the return value of one Hook:

```js 
export default function ChatRoom() {
  const [serverUrl, setServerUrl] = useState('https://localhost:1234');

  useChatRoom();
  // ...
```

and passing it as an input to another Hook:

```js 
export default function ChatRoom() {
  const [serverUrl, setServerUrl] = useState('https://localhost:1234');

  useChatRoom();
  // ...
```

Every time your `ChatRoom` component re-renders, it passes the latest `roomId` and `serverUrl` to your Hook. This is why your Effect re-connects to the chat whenever their values are different after a re-render. (If you ever worked with audio or video processing software, chaining Hooks like this might remind you of chaining visual or audio effects. It's as if the output of `useState` "feeds into" the input of the `useChatRoom`.)

### Passing event handlers to custom Hooks 

As you start using `useChatRoom` in more components, you might want to let components customize its behavior. For example, currently, the logic for what to do when a message arrives is hardcoded inside the Hook:

```js 
export function useChatRoom() {
  useEffect(() => {
    const options = ;
    const connection = createConnection(options);
    connection.connect();
    connection.on('message', (msg) => );
    return () => connection.disconnect();
  }, [roomId, serverUrl]);
}
```

Let's say you want to move this logic back to your component:

```js 
export default function ChatRoom() {
  const [serverUrl, setServerUrl] = useState('https://localhost:1234');

  useChatRoom({
    roomId: roomId,
    serverUrl: serverUrl,
    onReceiveMessage(msg) 
  });
  // ...
```

To make this work, change your custom Hook to take `onReceiveMessage` as one of its named options:

```js 
export function useChatRoom() {
  useEffect(() => {
    const options = ;
    const connection = createConnection(options);
    connection.connect();
    connection.on('message', (msg) => );
    return () => connection.disconnect();
  }, [roomId, serverUrl, onReceiveMessage]); // ✅ All dependencies declared
}
```

This will work, but there's one more improvement you can do when your custom Hook accepts event handlers.

Adding a dependency on `onReceiveMessage` is not ideal because it will cause the chat to re-connect every time the component re-renders. [Wrap this event handler into an Effect Event to remove it from the dependencies:](/learn/removing-effect-dependencies#wrapping-an-event-handler-from-the-props)

```js 
import  from 'react';
// ...

export function useChatRoom() {
  const onMessage = useEffectEvent(onReceiveMessage);

  useEffect(() => {
    const options = ;
    const connection = createConnection(options);
    connection.connect();
    connection.on('message', (msg) => );
    return () => connection.disconnect();
  }, [roomId, serverUrl]); // ✅ All dependencies declared
}
```

Now the chat won't re-connect every time that the `ChatRoom` component re-renders. Here is a fully working demo of passing an event handler to a custom Hook that you can play with:

Notice how you no longer need to know *how* `useChatRoom` works in order to use it. You could add it to any other component, pass any other options, and it would work the same way. That's the power of custom Hooks.

## When to use custom Hooks 

You don't need to extract a custom Hook for every little duplicated bit of code. Some duplication is fine. For example, extracting a `useFormInput` Hook to wrap a single `useState` call like earlier is probably unnecessary.

However, whenever you write an Effect, consider whether it would be clearer to also wrap it in a custom Hook. [You shouldn't need Effects very often,](/learn/you-might-not-need-an-effect) so if you're writing one, it means that you need to "step outside React" to synchronize with some external system or to do something that React doesn't have a built-in API for. Wrapping it into a custom Hook lets you precisely communicate your intent and how the data flows through it.

For example, consider a `ShippingForm` component that displays two dropdowns: one shows the list of cities, and another shows the list of areas in the selected city. You might start with some code that looks like this:

```js 
function ShippingForm() {
  const [cities, setCities] = useState(null);
  // This Effect fetches cities for a country
  useEffect(() => {
    let ignore = false;
    fetch(`/api/cities?country=$`)
      .then(response => response.json())
      .then(json => {
        if (!ignore) 
      });
    return () => ;
  }, [country]);

  const [city, setCity] = useState(null);
  const [areas, setAreas] = useState(null);
  // This Effect fetches areas for the selected city
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
  }, [city]);

  // ...
```

Although this code is quite repetitive, [it's correct to keep these Effects separate from each other.](/learn/removing-effect-dependencies#is-your-effect-doing-several-unrelated-things) They synchronize two different things, so you shouldn't merge them into one Effect. Instead, you can simplify the `ShippingForm` component above by extracting the common logic between them into your own `useData` Hook:

```js 
function useData(url) {
  const [data, setData] = useState(null);
  useEffect(() => {
    if (url) {
      let ignore = false;
      fetch(url)
        .then(response => response.json())
        .then(json => {
          if (!ignore) 
        });
      return () => ;
    }
  }, [url]);
  return data;
}
```

Now you can replace both Effects in the `ShippingForm` components with calls to `useData`:

```js 
function ShippingForm() {
  const cities = useData(`/api/cities?country=$`);
  const [city, setCity] = useState(null);
  const areas = useData(city ? `/api/areas?city=$` : null);
  // ...
```

Extracting a custom Hook makes the data flow explicit. You feed the `url` in and you get the `data` out. By "hiding" your Effect inside `useData`, you also prevent someone working on the `ShippingForm` component from adding [unnecessary dependencies](/learn/removing-effect-dependencies) to it. With time, most of your app's Effects will be in custom Hooks.

### Custom Hooks help you migrate to better patterns 

Effects are an ["escape hatch"](/learn/escape-hatches): you use them when you need to "step outside React" and when there is no better built-in solution for your use case. With time, the React team's goal is to reduce the number of the Effects in your app to the minimum by providing more specific solutions to more specific problems. Wrapping your Effects in custom Hooks makes it easier to upgrade your code when these solutions become available.

Let's return to this example:

In the above example, `useOnlineStatus` is implemented with a pair of [`useState`](/reference/react/useState) and [`useEffect`.](/reference/react/useEffect) However, this isn't the best possible solution. There is a number of edge cases it doesn't consider. For example, it assumes that when the component mounts, `isOnline` is already `true`, but this may be wrong if the network already went offline. You can use the browser [`navigator.onLine`](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/onLine) API to check for that, but using it directly would not work on the server for generating the initial HTML. In short, this code could be improved.

React includes a dedicated API called [`useSyncExternalStore`](/reference/react/useSyncExternalStore) which takes care of all of these problems for you. Here is your `useOnlineStatus` Hook, rewritten to take advantage of this new API:

Notice how **you didn't need to change any of the components** to make this migration:

```js 
function StatusBar() 

function SaveButton() 
```

This is another reason for why wrapping Effects in custom Hooks is often beneficial:

1. You make the data flow to and from your Effects very explicit.
2. You let your components focus on the intent rather than on the exact implementation of your Effects.
3. When React adds new features, you can remove those Effects without changing any of your components.

Similar to a [design system,](https://uxdesign.cc/everything-you-need-to-know-about-design-systems-54b109851969) you might find it helpful to start extracting common idioms from your app's components into custom Hooks. This will keep your components' code focused on the intent, and let you avoid writing raw Effects very often. Many excellent custom Hooks are maintained by the React community.

  );
}
```

We're still working out the details, but we expect that in the future, you'll write data fetching like this:

```js 
import  from 'react';

function ShippingForm() {
  const cities = use(fetch(`/api/cities?country=$`));
  const [city, setCity] = useState(null);
  const areas = city ? use(fetch(`/api/areas?city=$`)) : null;
  // ...
```

If you use custom Hooks like `useData` above in your app, it will require fewer changes to migrate to the eventually recommended approach than if you write raw Effects in every component manually. However, the old approach will still work fine, so if you feel happy writing raw Effects, you can continue to do that.

### There is more than one way to do it 

Let's say you want to implement a fade-in animation *from scratch* using the browser [`requestAnimationFrame`](https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame) API. You might start with an Effect that sets up an animation loop. During each frame of the animation, you could change the opacity of the DOM node you [hold in a ref](/learn/manipulating-the-dom-with-refs) until it reaches `1`. Your code might start like this:

To make the component more readable, you might extract the logic into a `useFadeIn` custom Hook:

You could keep the `useFadeIn` code as is, but you could also refactor it more. For example, you could extract the logic for setting up the animation loop out of `useFadeIn` into a custom `useAnimationLoop` Hook:

However, you didn't *have to* do that. As with regular functions, ultimately you decide where to draw the boundaries between different parts of your code. You could also take a very different approach. Instead of keeping the logic in the Effect, you could move most of the imperative logic inside a JavaScript [class:](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes)

Effects let you connect React to external systems. The more coordination between Effects is needed (for example, to chain multiple animations), the more it makes sense to extract that logic out of Effects and Hooks *completely* like in the sandbox above. Then, the code you extracted *becomes* the "external system". This lets your Effects stay simple because they only need to send messages to the system you've moved outside React.

The examples above assume that the fade-in logic needs to be written in JavaScript. However, this particular fade-in animation is both simpler and much more efficient to implement with a plain [CSS Animation:](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Animations/Using_CSS_animations)

Sometimes, you don't even need a Hook!

Notice that `App.js` doesn't need to import `useState` or `useEffect` anymore.

#### Make the counter delay configurable 

In this example, there is a `delay` state variable controlled by a slider, but its value is not used. Pass the `delay` value to your custom `useCounter` Hook, and change the `useCounter` Hook to use the passed `delay` instead of hardcoding `1000` ms.

#### Extract `useInterval` out of `useCounter` 

Currently, your `useCounter` Hook does two things. It sets up an interval, and it also increments a state variable on every interval tick. Split out the logic that sets up the interval into a separate Hook called `useInterval`. It should take two arguments: the `onTick` callback, and the `delay`. After this change, your `useCounter` implementation should look like this:

```js
export function useCounter(delay) {
  const [count, setCount] = useState(0);
  useInterval(() => , delay);
  return count;
}
```

Write `useInterval` in the `useInterval.js` file and import it into the `useCounter.js` file.

Note that there is a bit of a problem with this solution, which you'll solve in the next challenge.

#### Fix a resetting interval 

In this example, there are *two* separate intervals.

The `App` component calls `useCounter`, which calls `useInterval` to update the counter every second. But the `App` component *also* calls `useInterval` to randomly update the page background color every two seconds.

For some reason, the callback that updates the page background never runs. Add some logs inside `useInterval`:

```js 
  useEffect(() => {
    console.log('✅ Setting up an interval with delay ', delay)
    const id = setInterval(onTick, delay);
    return () => ;
  }, [onTick, delay]);
```

Do the logs match what you expect to happen? If some of your Effects seem to re-synchronize unnecessarily, can you guess which dependency is causing that to happen? Is there some way to [remove that dependency](/learn/removing-effect-dependencies) from your Effect?

After you fix the issue, you should expect the page background to update every two seconds.

#### Implement a staggering movement 

In this example, the `usePointerPosition()` Hook tracks the current pointer position. Try moving your cursor or your finger over the preview area and see the red dot follow your movement. Its position is saved in the `pos1` variable.

In fact, there are five (!) different red dots being rendered. You don't see them because currently they all appear at the same position. This is what you need to fix. What you want to implement instead is a "staggered" movement: each dot should "follow" the previous dot's path. For example, if you quickly move your cursor, the first dot should follow it immediately, the second dot should follow the first dot with a small delay, the third dot should follow the second dot, and so on.

You need to implement the `useDelayedValue` custom Hook. Its current implementation returns the `value` provided to it. Instead, you want to return the value back from `delay` milliseconds ago. You might need some state and an Effect to do this.

After you implement `useDelayedValue`, you should see the dots move following one another.

Note that this Effect *does not* need cleanup. If you called `clearTimeout` in the cleanup function, then each time the `value` changes, it would reset the already scheduled timeout. To keep the movement continuous, you want all the timeouts to fire.

