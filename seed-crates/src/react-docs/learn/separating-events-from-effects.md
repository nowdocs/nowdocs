---
title: 'Separating Events from Effects'
---

## Choosing between event handlers and Effects 

First, let's recap the difference between event handlers and Effects.

Imagine you're implementing a chat room component. Your requirements look like this:

1. Your component should automatically connect to the selected chat room.
1. When you click the "Send" button, it should send a message to the chat.

Let's say you've already implemented the code for them, but you're not sure where to put it. Should you use event handlers or Effects? Every time you need to answer this question, consider [*why* the code needs to run.](/learn/synchronizing-with-effects#what-are-effects-and-how-are-they-different-from-events)

### Event handlers run in response to specific interactions 

From the user's perspective, sending a message should happen *because* the particular "Send" button was clicked. The user will get rather upset if you send their message at any other time or for any other reason. This is why sending a message should be an event handler. Event handlers let you handle specific interactions:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');
  // ...
  function handleSendClick() 
  // ...
  return (
    <>
       setMessage(e.target.value)} />
      Send
    </>
  );
}
```

With an event handler, you can be sure that `sendMessage(message)` will *only* run if the user presses the button.

### Effects run whenever synchronization is needed 

Recall that you also need to keep the component connected to the chat room. Where does that code go?

The *reason* to run this code is not some particular interaction. It doesn't matter why or how the user navigated to the chat room screen. Now that they're looking at it and could interact with it, the component needs to stay connected to the selected chat server. Even if the chat room component was the initial screen of your app, and the user has not performed any interactions at all, you would *still* need to connect. This is why it's an Effect:

```js 
function ChatRoom() {
  // ...
  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.connect();
    return () => ;
  }, [roomId]);
  // ...
}
```

With this code, you can be sure that there is always an active connection to the currently selected chat server, *regardless* of the specific interactions performed by the user. Whether the user has only opened your app, selected a different room, or navigated to another screen and back, your Effect ensures that the component will *remain synchronized* with the currently selected room, and will [re-connect whenever it's necessary.](/learn/lifecycle-of-reactive-effects#why-synchronization-may-need-to-happen-more-than-once)

## Reactive values and reactive logic 

Intuitively, you could say that event handlers are always triggered "manually", for example by clicking a button. Effects, on the other hand, are "automatic": they run and re-run as often as it's needed to stay synchronized.

There is a more precise way to think about this.

Props, state, and variables declared inside your component's body are called . In this example, `serverUrl` is not a reactive value, but `roomId` and `message` are. They participate in the rendering data flow:

```js [[2, 3, "roomId"], [2, 4, "message"]]
const serverUrl = 'https://localhost:1234';

function ChatRoom() 
```

Reactive values like these can change due to a re-render. For example, the user may edit the `message` or choose a different `roomId` in a dropdown. Event handlers and Effects respond to changes differently:

- **Logic inside event handlers is *not reactive.*** It will not run again unless the user performs the same interaction (e.g. a click) again. Event handlers can read reactive values without "reacting" to their changes.
- **Logic inside Effects is *reactive.*** If your Effect reads a reactive value, [you have to specify it as a dependency.](/learn/lifecycle-of-reactive-effects#effects-react-to-reactive-values) Then, if a re-render causes that value to change, React will re-run your Effect's logic with the new value.

Let's revisit the previous example to illustrate this difference.

### Logic inside event handlers is not reactive 

Take a look at this line of code. Should this logic be reactive or not?

```js [[2, 2, "message"]]
    // ...
    sendMessage(message);
    // ...
```

From the user's perspective, **a change to the `message` does _not_ mean that they want to send a message.** It only means that the user is typing. In other words, the logic that sends a message should not be reactive. It should not run again only because the  has changed. That's why it belongs in the event handler:

```js 
  function handleSendClick() 
```

Event handlers aren't reactive, so `sendMessage(message)` will only run when the user clicks the Send button.

### Logic inside Effects is reactive 

Now let's return to these lines:

```js [[2, 2, "roomId"]]
    // ...
    const connection = createConnection(serverUrl, roomId);
    connection.connect();
    // ...
```

From the user's perspective, **a change to the `roomId` *does* mean that they want to connect to a different room.** In other words, the logic for connecting to the room should be reactive. You *want* these lines of code to "keep up" with the , and to run again if that value is different. That's why it belongs in an Effect:

```js 
  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.connect();
    return () => ;
  }, [roomId]);
```

Effects are reactive, so `createConnection(serverUrl, roomId)` and `connection.connect()` will run for every distinct value of `roomId`. Your Effect keeps the chat connection synchronized to the currently selected room.

## Extracting non-reactive logic out of Effects 

Things get more tricky when you want to mix reactive logic with non-reactive logic.

For example, imagine that you want to show a notification when the user connects to the chat. You read the current theme (dark or light) from the props so that you can show the notification in the correct color:

```js 
function ChatRoom() {
  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.on('connected', () => );
    connection.connect();
    // ...
```

However, `theme` is a reactive value (it can change as a result of re-rendering), and [every reactive value read by an Effect must be declared as its dependency.](/learn/lifecycle-of-reactive-effects#react-verifies-that-you-specified-every-reactive-value-as-a-dependency) Now you have to specify `theme` as a dependency of your Effect:

```js 
function ChatRoom() {
  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.on('connected', () => );
    connection.connect();
    return () => ;
  }, [roomId, theme]); // ✅ All dependencies declared
  // ...
```

Play with this example and see if you can spot the problem with this user experience:

When the `roomId` changes, the chat re-connects as you would expect. But since `theme` is also a dependency, the chat *also* re-connects every time you switch between the dark and the light theme. That's not great!

In other words, you *don't* want this line to be reactive, even though it is inside an Effect (which is reactive):

```js
      // ...
      showNotification('Connected!', theme);
      // ...
```

You need a way to separate this non-reactive logic from the reactive Effect around it.

### Declaring an Effect Event 

Use a special Hook called [`useEffectEvent`](/reference/react/useEffectEvent) to extract this non-reactive logic out of your Effect:

```js 
import  from 'react';

function ChatRoom() {
  const onConnected = useEffectEvent(() => );
  // ...
```

Here, `onConnected` is called an *Effect Event.* It's a part of your Effect logic, but it behaves a lot more like an event handler. The logic inside it is not reactive, and it always "sees" the latest values of your props and state.

Now you can call the `onConnected` Effect Event from inside your Effect:

```js 
function ChatRoom() {
  const onConnected = useEffectEvent(() => );

  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.on('connected', () => );
    connection.connect();
    return () => connection.disconnect();
  }, [roomId]); // ✅ All dependencies declared
  // ...
```

This solves the problem. Note that you had to *remove* `theme` from the list of your Effect's dependencies, because it's no longer used in the Effect. You also don't need to *add* `onConnected` to it, because **Effect Events are not reactive and must be omitted from dependencies.**

Verify that the new behavior works as you would expect:

You can think of Effect Events as being very similar to event handlers. The main difference is that event handlers run in response to user interactions, whereas Effect Events are triggered by you from Effects. Effect Events let you "break the chain" between the reactivity of Effects and code that should not be reactive.

### Reading latest props and state with Effect Events 

Effect Events let you fix many patterns where you might be tempted to suppress the dependency linter.

For example, say you have an Effect to log the page visits:

```js
function Page() {
  useEffect(() => , []);
  // ...
}
```

Later, you add multiple routes to your site. Now your `Page` component receives a `url` prop with the current path. You want to pass the `url` as a part of your `logVisit` call, but the dependency linter complains:

```js 
function Page() {
  useEffect(() => , []); // 🔴 React Hook useEffect has a missing dependency: 'url'
  // ...
}
```

Think about what you want the code to do. You *want* to log a separate visit for different URLs since each URL represents a different page. In other words, this `logVisit` call *should* be reactive with respect to the `url`. This is why, in this case, it makes sense to follow the dependency linter, and add `url` as a dependency:

```js 
function Page() {
  useEffect(() => , [url]); // ✅ All dependencies declared
  // ...
}
```

Now let's say you want to include the number of items in the shopping cart together with every page visit:

```js 
function Page() {
  const  = useContext(ShoppingCartContext);
  const numberOfItems = items.length;

  useEffect(() => , [url]); // 🔴 React Hook useEffect has a missing dependency: 'numberOfItems'
  // ...
}
```

You used `numberOfItems` inside the Effect, so the linter asks you to add it as a dependency. However, you *don't* want the `logVisit` call to be reactive with respect to `numberOfItems`. If the user puts something into the shopping cart, and the `numberOfItems` changes, this *does not mean* that the user visited the page again. In other words, *visiting the page* is, in some sense, an "event". It happens at a precise moment in time.

Split the code in two parts:

```js 
function Page() {
  const  = useContext(ShoppingCartContext);
  const numberOfItems = items.length;

  const onVisit = useEffectEvent(visitedUrl => );

  useEffect(() => , [url]); // ✅ All dependencies declared
  // ...
}
```

Here, `onVisit` is an Effect Event. The code inside it isn't reactive. This is why you can use `numberOfItems` (or any other reactive value!) without worrying that it will cause the surrounding code to re-execute on changes.

On the other hand, the Effect itself remains reactive. Code inside the Effect uses the `url` prop, so the Effect will re-run after every re-render with a different `url`. This, in turn, will call the `onVisit` Effect Event.

As a result, you will call `logVisit` for every change to the `url`, and always read the latest `numberOfItems`. However, if `numberOfItems` changes on its own, this will not cause any of the code to re-run.

The problem with this code is in suppressing the dependency linter. If you remove the suppression, you'll see that this Effect should depend on the `handleMove` function. This makes sense: `handleMove` is declared inside the component body, which makes it a reactive value. Every reactive value must be specified as a dependency, or it can potentially get stale over time!

The author of the original code has "lied" to React by saying that the Effect does not depend (`[]`) on any reactive values. This is why React did not re-synchronize the Effect after `canMove` has changed (and `handleMove` with it). Because React did not re-synchronize the Effect, the `handleMove` attached as a listener is the `handleMove` function created during the initial render. During the initial render, `canMove` was `true`, which is why `handleMove` from the initial render will forever see that value.

**If you never suppress the linter, you will never see problems with stale values.**

With `useEffectEvent`, there is no need to "lie" to the linter, and the code works as you would expect:

This doesn't mean that `useEffectEvent` is *always* the correct solution. You should only apply it to the lines of code that you don't want to be reactive. In the above sandbox, you didn't want the Effect's code to be reactive with regards to `canMove`. That's why it made sense to extract an Effect Event.

Read [Removing Effect Dependencies](/learn/removing-effect-dependencies) for other correct alternatives to suppressing the linter.

### Limitations of Effect Events 

Effect Events are very limited in how you can use them:

* **Only call them from inside Effects.**
* **Never pass them to other components or Hooks.**

For example, don't declare and pass an Effect Event like this:

```js 
function Timer() {
  const [count, setCount] = useState(0);

  const onTick = useEffectEvent(() => );

  useTimer(onTick, 1000); // 🔴 Avoid: Passing Effect Events

  return 
}

function useTimer(callback, delay) {
  useEffect(() => {
    const id = setInterval(() => , delay);
    return () => ;
  }, [delay, callback]); // Need to specify "callback" in dependencies
}
```

Instead, always declare Effect Events directly next to the Effects that use them:

```js 
function Timer() {
  const [count, setCount] = useState(0);
  useTimer(() => , 1000);
  return 
}

function useTimer(callback, delay) {
  const onTick = useEffectEvent(() => );

  useEffect(() => {
    const id = setInterval(() => , delay);
    return () => ;
  }, [delay]); // No need to specify "onTick" (an Effect Event) as a dependency
}
```

Effect Events are non-reactive "pieces" of your Effect code. They should be next to the Effect using them.

Now, when `increment` changes, React will re-synchronize your Effect, which will restart the interval.

#### Fix a freezing counter 

This `Timer` component keeps a `count` state variable which increases every second. The value by which it's increasing is stored in the `increment` state variable, which you can control it with the plus and minus buttons. For example, try pressing the plus button nine times, and notice that the `count` now increases each second by ten rather than by one.

There is a small issue with this user interface. You might notice that if you keep pressing the plus or minus buttons faster than once per second, the timer itself seems to pause. It only resumes after a second passes since the last time you've pressed either button. Find why this is happening, and fix the issue so that the timer ticks on *every* second without interruptions.

Since `onTick` is an Effect Event, the code inside it isn't reactive. The change to `increment` does not trigger any Effects.

#### Fix a non-adjustable delay 

In this example, you can customize the interval delay. It's stored in a `delay` state variable which is updated by two buttons. However, even if you press the "plus 100 ms" button until the `delay` is 1000 milliseconds (that is, a second), you'll notice that the timer still increments very fast (every 100 ms). It's as if your changes to the `delay` are ignored. Find and fix the bug.

In general, you should be suspicious of functions like `onMount` that focus on the *timing* rather than the *purpose* of a piece of code. It may feel "more descriptive" at first but it obscures your intent. As a rule of thumb, Effect Events should correspond to something that happens from the *user's* perspective. For example, `onMessage`, `onTick`, `onVisit`, or `onConnected` are good Effect Event names. Code inside them would likely not need to be reactive. On the other hand, `onMount`, `onUpdate`, `onUnmount`, or `onAfterRender` are so generic that it's easy to accidentally put code that *should* be reactive into them. This is why you should name your Effect Events after *what the user thinks has happened,* not when some code happened to run.

#### Fix a delayed notification 

When you join a chat room, this component shows a notification. However, it doesn't show the notification immediately. Instead, the notification is artificially delayed by two seconds so that the user has a chance to look around the UI.

This almost works, but there is a bug. Try changing the dropdown from "general" to "travel" and then to "music" very quickly. If you do it fast enough, you will see two notifications (as expected!) but they will *both* say "Welcome to music".

Fix it so that when you switch from "general" to "travel" and then to "music" very quickly, you see two notifications, the first one being "Welcome to travel" and the second one being "Welcome to music". (For an additional challenge, assuming you've *already* made the notifications show the correct rooms, change the code so that only the latter notification is displayed.)

The Effect that had `roomId` set to `"travel"` (so it connected to the `"travel"` room) will show the notification for `"travel"`. The Effect that had `roomId` set to `"music"` (so it connected to the `"music"` room) will show the notification for `"music"`. In other words, `connectedRoomId` comes from your Effect (which is reactive), while `theme` always uses the latest value.

To solve the additional challenge, save the notification timeout ID and clear it in the cleanup function of your Effect:

This ensures that already scheduled (but not yet displayed) notifications get cancelled when you change rooms.

