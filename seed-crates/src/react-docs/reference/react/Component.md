---
title: Component
---

---

### `props` 

The props passed to a class component are available as `this.props`.

```js 
class Greeting extends Component {
  render() {
    return Hello, !;
  }
}

---

### `state` 

The state of a class component is available as `this.state`. The `state` field must be an object. Do not mutate the state directly. If you wish to change the state, call `setState` with the new state.

```js 
class Counter extends Component {
  state = ;

  handleAgeChange = () => {
    this.setState();
  };

  render() {
    return (
      <>
        
        Increment age
        
        You are .
      </>
    );
  }
}
```

---

### `constructor(props)` 

The [constructor](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/constructor) runs before your class component *mounts* (gets added to the screen). Typically, a constructor is only used for two purposes in React. It lets you declare state and [bind](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_objects/Function/bind) your class methods to the class instance:

```js 
class Counter extends Component {
  constructor(props) {
    super(props);
    this.state = ;
    this.handleClick = this.handleClick.bind(this);
  }

  handleClick() 
```

If you use modern JavaScript syntax, constructors are rarely needed. Instead, you can rewrite this code above using the [public class field syntax](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Public_class_fields) which is supported both by modern browsers and tools like [Babel:](https://babeljs.io/)

```js 
class Counter extends Component {
  state = ;

  handleClick = () => 
```

A constructor should not contain any side effects or subscriptions.

#### Parameters 

* `props`: The component's initial props.

#### Returns 

`constructor` should not return anything.

#### Caveats 

* Do not run any side effects or subscriptions in the constructor. Instead, use [`componentDidMount`](#componentdidmount) for that.

* Inside a constructor, you need to call `super(props)` before any other statement. If you don't do that, `this.props` will be `undefined` while the constructor runs, which can be confusing and cause bugs.

* Constructor is the only place where you can assign [`this.state`](#state) directly. In all other methods, you need to use [`this.setState()`](#setstate) instead. Do not call `setState` in the constructor.

* When you use [server rendering,](/reference/react-dom/server) the constructor will run on the server too, followed by the [`render`](#render) method. However, lifecycle methods like `componentDidMount` or `componentWillUnmount` will not run on the server.

* When [Strict Mode](/reference/react/StrictMode) is on, React will call `constructor` twice in development and then throw away one of the instances. This helps you notice the accidental side effects that need to be moved out of the `constructor`.

---

### `componentDidCatch(error, info)` 

If you define `componentDidCatch`, React will call it when some child component (including distant children) throws an error during rendering. This lets you log that error to an error reporting service in production.

Typically, it is used together with [`static getDerivedStateFromError`](#static-getderivedstatefromerror) which lets you update state in response to an error and display an error message to the user. A component with these methods is called an *Error Boundary*.

[See an example.](#catching-rendering-errors-with-an-error-boundary)

#### Parameters 

* `error`: The error that was thrown. In practice, it will usually be an instance of [`Error`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error) but this is not guaranteed because JavaScript allows to [`throw`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw) any value, including strings or even `null`.

* `info`: An object containing additional information about the error. Its `componentStack` field contains a stack trace with the component that threw, as well as the names and source locations of all its parent components. In production, the component names will be minified. If you set up production error reporting, you can decode the component stack using sourcemaps the same way as you would do for regular JavaScript error stacks.

#### Returns 

`componentDidCatch` should not return anything.

#### Caveats 

* In the past, it was common to call `setState` inside `componentDidCatch` in order to update the UI and display the fallback error message. This is deprecated in favor of defining [`static getDerivedStateFromError`.](#static-getderivedstatefromerror)

* Production and development builds of React slightly differ in the way `componentDidCatch` handles errors. In development, the errors will bubble up to `window`, which means that any `window.onerror` or `window.addEventListener('error', callback)` will intercept the errors that have been caught by `componentDidCatch`. In production, instead, the errors will not bubble up, which means any ancestor error handler will only receive errors not explicitly caught by `componentDidCatch`.

---

### `componentDidMount()` 

If you define the `componentDidMount` method, React will call it when your component is added *(mounted)* to the screen. This is a common place to start data fetching, set up subscriptions, or manipulate the DOM nodes.

If you implement `componentDidMount`, you usually need to implement other lifecycle methods to avoid bugs. For example, if `componentDidMount` reads some state or props, you also have to implement [`componentDidUpdate`](#componentdidupdate) to handle their changes, and [`componentWillUnmount`](#componentwillunmount) to clean up whatever `componentDidMount` was doing.

```js 
class ChatRoom extends Component {
  state = ;

  componentDidMount() 

  componentDidUpdate(prevProps, prevState) {
    if (
      this.props.roomId !== prevProps.roomId ||
      this.state.serverUrl !== prevState.serverUrl
    ) 
  }

  componentWillUnmount() 

  // ...
}
```

[See more examples.](#adding-lifecycle-methods-to-a-class-component)

#### Parameters 

`componentDidMount` does not take any parameters.

#### Returns 

`componentDidMount` should not return anything.

#### Caveats 

- When [Strict Mode](/reference/react/StrictMode) is on, in development React will call `componentDidMount`, then immediately call [`componentWillUnmount`,](#componentwillunmount) and then call `componentDidMount` again. This helps you notice if you forgot to implement `componentWillUnmount` or if its logic doesn't fully "mirror" what `componentDidMount` does.

- Although you may call [`setState`](#setstate) immediately in `componentDidMount`, it's best to avoid that when you can. It will trigger an extra rendering, but it will happen before the browser updates the screen. This guarantees that even though the [`render`](#render) will be called twice in this case, the user won't see the intermediate state. Use this pattern with caution because it often causes performance issues. In most cases, you should be able to assign the initial state in the [`constructor`](#constructor) instead. It can, however, be necessary for cases like modals and tooltips when you need to measure a DOM node before rendering something that depends on its size or position.

---

### `componentDidUpdate(prevProps, prevState, snapshot?)` 

If you define the `componentDidUpdate` method, React will call it immediately after your component has been re-rendered with updated props or state.  This method is not called for the initial render.

You can use it to manipulate the DOM after an update. This is also a common place to do network requests as long as you compare the current props to previous props (e.g. a network request may not be necessary if the props have not changed). Typically, you'd use it together with [`componentDidMount`](#componentdidmount) and [`componentWillUnmount`:](#componentwillunmount)

```js 
class ChatRoom extends Component {
  state = ;

  componentDidMount() 

  componentDidUpdate(prevProps, prevState) {
    if (
      this.props.roomId !== prevProps.roomId ||
      this.state.serverUrl !== prevState.serverUrl
    ) 
  }

  componentWillUnmount() 

  // ...
}
```

[See more examples.](#adding-lifecycle-methods-to-a-class-component)

#### Parameters 

* `prevProps`: Props before the update. Compare `prevProps` to [`this.props`](#props) to determine what changed.

* `prevState`: State before the update. Compare `prevState` to [`this.state`](#state) to determine what changed.

* `snapshot`: If you implemented [`getSnapshotBeforeUpdate`](#getsnapshotbeforeupdate), `snapshot` will contain the value you returned from that method. Otherwise, it will be `undefined`.

#### Returns 

`componentDidUpdate` should not return anything.

#### Caveats 

- `componentDidUpdate` will not get called if [`shouldComponentUpdate`](#shouldcomponentupdate) is defined and returns `false`.

- The logic inside `componentDidUpdate` should usually be wrapped in conditions comparing `this.props` with `prevProps`, and `this.state` with `prevState`. Otherwise, there's a risk of creating infinite loops.

- Although you may call [`setState`](#setstate) immediately in `componentDidUpdate`, it's best to avoid that when you can. It will trigger an extra rendering, but it will happen before the browser updates the screen. This guarantees that even though the [`render`](#render) will be called twice in this case, the user won't see the intermediate state. This pattern often causes performance issues, but it may be necessary for rare cases like modals and tooltips when you need to measure a DOM node before rendering something that depends on its size or position.

---

### `componentWillMount()` 

---

### `componentWillReceiveProps(nextProps)` 

---

### `componentWillUpdate(nextProps, nextState)` 

---

### `componentWillUnmount()` 

If you define the `componentWillUnmount` method, React will call it before your component is removed *(unmounted)* from the screen. This is a common place to cancel data fetching or remove subscriptions.

The logic inside `componentWillUnmount` should "mirror" the logic inside [`componentDidMount`.](#componentdidmount) For example, if `componentDidMount` sets up a subscription, `componentWillUnmount` should clean up that subscription. If the cleanup logic in your `componentWillUnmount` reads some props or state, you will usually also need to implement [`componentDidUpdate`](#componentdidupdate) to clean up resources (such as subscriptions) corresponding to the old props and state.

```js 
class ChatRoom extends Component {
  state = ;

  componentDidMount() 

  componentDidUpdate(prevProps, prevState) {
    if (
      this.props.roomId !== prevProps.roomId ||
      this.state.serverUrl !== prevState.serverUrl
    ) 
  }

  componentWillUnmount() 

  // ...
}
```

[See more examples.](#adding-lifecycle-methods-to-a-class-component)

#### Parameters 

`componentWillUnmount` does not take any parameters.

#### Returns 

`componentWillUnmount` should not return anything.

#### Caveats 

- When [Strict Mode](/reference/react/StrictMode) is on, in development React will call [`componentDidMount`,](#componentdidmount) then immediately call `componentWillUnmount`, and then call `componentDidMount` again. This helps you notice if you forgot to implement `componentWillUnmount` or if its logic doesn't fully "mirror" what `componentDidMount` does.

---

### `forceUpdate(callback?)` 

Forces a component to re-render.

Usually, this is not necessary. If your component's [`render`](#render) method only reads from [`this.props`](#props), [`this.state`](#state), or [`this.context`,](#context) it will re-render automatically when you call [`setState`](#setstate) inside your component or one of its parents. However, if your component's `render` method reads directly from an external data source, you have to tell React to update the user interface when that data source changes. That's what `forceUpdate` lets you do.

Try to avoid all uses of `forceUpdate` and only read from `this.props` and `this.state` in `render`.

#### Parameters 

* **optional** `callback` If specified, React will call the `callback` you've provided after the update is committed.

#### Returns 

`forceUpdate` does not return anything.

#### Caveats 

- If you call `forceUpdate`, React will re-render without calling [`shouldComponentUpdate`.](#shouldcomponentupdate)

---

### `getSnapshotBeforeUpdate(prevProps, prevState)` 

If you implement `getSnapshotBeforeUpdate`, React will call it immediately before React updates the DOM. It enables your component to capture some information from the DOM (e.g. scroll position) before it is potentially changed. Any value returned by this lifecycle method will be passed as a parameter to [`componentDidUpdate`.](#componentdidupdate)

For example, you can use it in a UI like a chat thread that needs to preserve its scroll position during updates:

```js 
class ScrollingList extends React.Component {
  constructor(props) 

  getSnapshotBeforeUpdate(prevProps, prevState) {
    // Are we adding new items to the list?
    // Capture the scroll position so we can adjust scroll later.
    if (prevProps.list.length < this.props.list.length) 
    return null;
  }

  componentDidUpdate(prevProps, prevState, snapshot) {
    // If we have a snapshot value, we've just added new items.
    // Adjust scroll so these new items don't push the old ones out of view.
    // (snapshot here is the value returned from getSnapshotBeforeUpdate)
    if (snapshot !== null) 
  }

  render() {
    return (
      
    );
  }
}
```

In the above example, it is important to read the `scrollHeight` property directly in `getSnapshotBeforeUpdate`. It is not safe to read it in [`render`](#render), [`UNSAFE_componentWillReceiveProps`](#unsafe_componentwillreceiveprops), or [`UNSAFE_componentWillUpdate`](#unsafe_componentwillupdate) because there is a potential time gap between these methods getting called and React updating the DOM.

#### Parameters 

* `prevProps`: Props before the update. Compare `prevProps` to [`this.props`](#props) to determine what changed.

* `prevState`: State before the update. Compare `prevState` to [`this.state`](#state) to determine what changed.

#### Returns 

You should return a snapshot value of any type that you'd like, or `null`. The value you returned will be passed as the third argument to [`componentDidUpdate`.](#componentdidupdate)

#### Caveats 

- `getSnapshotBeforeUpdate` will not get called if [`shouldComponentUpdate`](#shouldcomponentupdate) is defined and returns `false`.

---

### `render()` 

The `render` method is the only required method in a class component.

The `render` method should specify what you want to appear on the screen, for example:

```js 
import  from 'react';

class Greeting extends Component {
  render() {
    return Hello, !;
  }
}
```

React may call `render` at any moment, so you shouldn't assume that it runs at a particular time. Usually, the `render` method should return a piece of [JSX](/learn/writing-markup-with-jsx), but a few [other return types](#render-returns) (like strings) are supported. To calculate the returned JSX, the `render` method can read [`this.props`](#props), [`this.state`](#state), and [`this.context`](#context).

You should write the `render` method as a pure function, meaning that it should return the same result if props, state, and context are the same. It also shouldn't contain side effects (like setting up subscriptions) or interact with the browser APIs. Side effects should happen either in event handlers or methods like [`componentDidMount`.](#componentdidmount)

#### Parameters 

`render` does not take any parameters.

#### Returns 

`render` can return any valid React node. This includes React elements such as ``, strings, numbers, [portals](/reference/react-dom/createPortal), empty nodes (`null`, `undefined`, `true`, and `false`), and arrays of React nodes.

#### Caveats 

- `render` should be written as a pure function of props, state, and context. It should not have side effects.

- `render` will not get called if [`shouldComponentUpdate`](#shouldcomponentupdate) is defined and returns `false`.

- When [Strict Mode](/reference/react/StrictMode) is on, React will call `render` twice in development and then throw away one of the results. This helps you notice the accidental side effects that need to be moved out of the `render` method.

- There is no one-to-one correspondence between the `render` call and the subsequent `componentDidMount` or `componentDidUpdate` call. Some of the `render` call results may be discarded by React when it's beneficial.

---

### `setState(nextState, callback?)` 

Call `setState` to update the state of your React component.

```js 
class Form extends Component {
  state = ;

  handleNameChange = (e) => {
    const newName = e.target.value;
    this.setState();
  }

  render() {
    return (
      <>
        
        Hello, .
      </>
    );
  }
}
```

`setState` enqueues changes to the component state. It tells React that this component and its children need to re-render with the new state. This is the main way you'll update the user interface in response to interactions.

You can also pass a function to `setState`. It lets you update state based on the previous state:

```js 
  handleIncreaseAge = () => {
    this.setState(prevState => {
      return ;
    });
  }
```

You don't have to do this, but it's handy if you want to update state multiple times during the same event.

#### Parameters 

* `nextState`: Either an object or a function.
  * If you pass an object as `nextState`, it will be shallowly merged into `this.state`.
  * If you pass a function as `nextState`, it will be treated as an _updater function_. It must be pure, should take the pending state and props as arguments, and should return the object to be shallowly merged into `this.state`. React will put your updater function in a queue and re-render your component. During the next render, React will calculate the next state by applying all of the queued updaters to the previous state.

* **optional** `callback`: If specified, React will call the `callback` you've provided after the update is committed.

#### Returns 

`setState` does not return anything.

#### Caveats 

- Think of `setState` as a *request* rather than an immediate command to update the component. When multiple components update their state in response to an event, React will batch their updates and re-render them together in a single pass at the end of the event. In the rare case that you need to force a particular state update to be applied synchronously, you may wrap it in [`flushSync`,](/reference/react-dom/flushSync) but this may hurt performance.

- `setState` does not update `this.state` immediately. This makes reading `this.state` right after calling `setState` a potential pitfall. Instead, use [`componentDidUpdate`](#componentdidupdate) or the setState `callback` argument, either of which are guaranteed to fire after the update has been applied. If you need to set the state based on the previous state, you can pass a function to `nextState` as described above.

---

### `shouldComponentUpdate(nextProps, nextState, nextContext)` 

If you define `shouldComponentUpdate`, React will call it to determine whether a re-render can be skipped.

If you are confident you want to write it by hand, you may compare `this.props` with `nextProps` and `this.state` with `nextState` and return `false` to tell React the update can be skipped.

```js 
class Rectangle extends Component {
  state = ;

  shouldComponentUpdate(nextProps, nextState) {
    if (
      nextProps.position.x === this.props.position.x &&
      nextProps.position.y === this.props.position.y &&
      nextProps.size.width === this.props.size.width &&
      nextProps.size.height === this.props.size.height &&
      nextState.isHovered === this.state.isHovered
    ) 
    return true;
  }

  // ...
}

```

React calls `shouldComponentUpdate` before rendering when new props or state are being received. Defaults to `true`. This method is not called for the initial render or when [`forceUpdate`](#forceupdate) is used.

#### Parameters 

- `nextProps`: The next props that the component is about to render with. Compare `nextProps` to [`this.props`](#props) to determine what changed.
- `nextState`: The next state that the component is about to render with. Compare `nextState` to [`this.state`](#props) to determine what changed.
- `nextContext`: The next context that the component is about to render with. Compare `nextContext` to [`this.context`](#context) to determine what changed. Only available if you specify [`static contextType`](#static-contexttype).

#### Returns 

Return `true` if you want the component to re-render. That's the default behavior.

Return `false` to tell React that re-rendering can be skipped.

#### Caveats 

- This method *only* exists as a performance optimization. If your component breaks without it, fix that first.

- Consider using [`PureComponent`](/reference/react/PureComponent) instead of writing `shouldComponentUpdate` by hand. `PureComponent` shallowly compares props and state, and reduces the chance that you'll skip a necessary update.

- We do not recommend doing deep equality checks or using `JSON.stringify` in `shouldComponentUpdate`. It makes performance unpredictable and dependent on the data structure of every prop and state. In the best case, you risk introducing multi-second stalls to your application, and in the worst case you risk crashing it.

- Returning `false` does not prevent child components from re-rendering when *their* state changes.

- Returning `false` does not *guarantee* that the component will not re-render. React will use the return value as a hint but it may still choose to re-render your component if it makes sense to do for other reasons.

---

### `UNSAFE_componentWillMount()` 

If you define `UNSAFE_componentWillMount`, React will call it immediately after the [`constructor`.](#constructor) It only exists for historical reasons and should not be used in any new code. Instead, use one of the alternatives:

- To initialize state, declare [`state`](#state) as a class field or set `this.state` inside the [`constructor`.](#constructor)
- If you need to run a side effect or set up a subscription, move that logic to [`componentDidMount`](#componentdidmount) instead.

[See examples of migrating away from unsafe lifecycles.](https://legacy.reactjs.org/blog/2018/03/27/update-on-async-rendering.html#examples)

#### Parameters 

`UNSAFE_componentWillMount` does not take any parameters.

#### Returns 

`UNSAFE_componentWillMount` should not return anything.

#### Caveats 

- `UNSAFE_componentWillMount` will not get called if the component implements [`static getDerivedStateFromProps`](#static-getderivedstatefromprops) or [`getSnapshotBeforeUpdate`.](#getsnapshotbeforeupdate)

- Despite its naming, `UNSAFE_componentWillMount` does not guarantee that the component *will* get mounted if your app uses modern React features like [`Suspense`.](/reference/react/Suspense) If a render attempt is suspended (for example, because the code for some child component has not loaded yet), React will throw the in-progress tree away and attempt to construct the component from scratch during the next attempt. This is why this method is "unsafe". Code that relies on mounting (like adding a subscription) should go into [`componentDidMount`.](#componentdidmount)

- `UNSAFE_componentWillMount` is the only lifecycle method that runs during [server rendering.](/reference/react-dom/server) For all practical purposes, it is identical to [`constructor`,](#constructor) so you should use the `constructor` for this type of logic instead.

---

### `UNSAFE_componentWillReceiveProps(nextProps, nextContext)` 

If you define `UNSAFE_componentWillReceiveProps`, React will call it when the component receives new props. It only exists for historical reasons and should not be used in any new code. Instead, use one of the alternatives:

- If you need to **run a side effect** (for example, fetch data, run an animation, or reinitialize a subscription) in response to prop changes, move that logic to [`componentDidUpdate`](#componentdidupdate) instead.
- If you need to **avoid re-computing some data only when a prop changes,** use a [memoization helper](https://legacy.reactjs.org/blog/2018/06/07/you-probably-dont-need-derived-state.html#what-about-memoization) instead.
- If you need to **"reset" some state when a prop changes,** consider either making a component [fully controlled](https://legacy.reactjs.org/blog/2018/06/07/you-probably-dont-need-derived-state.html#recommendation-fully-controlled-component) or [fully uncontrolled with a key](https://legacy.reactjs.org/blog/2018/06/07/you-probably-dont-need-derived-state.html#recommendation-fully-uncontrolled-component-with-a-key) instead.
- If you need to **"adjust" some state when a prop changes,** check whether you can compute all the necessary information from props alone during rendering. If you can't, use [`static getDerivedStateFromProps`](/reference/react/Component#static-getderivedstatefromprops) instead.

[See examples of migrating away from unsafe lifecycles.](https://legacy.reactjs.org/blog/2018/03/27/update-on-async-rendering.html#updating-state-based-on-props)

#### Parameters 

- `nextProps`: The next props that the component is about to receive from its parent component. Compare `nextProps` to [`this.props`](#props) to determine what changed.
- `nextContext`: The next context that the component is about to receive from the closest provider. Compare `nextContext` to [`this.context`](#context) to determine what changed. Only available if you specify [`static contextType`](#static-contexttype).

#### Returns 

`UNSAFE_componentWillReceiveProps` should not return anything.

#### Caveats 

- `UNSAFE_componentWillReceiveProps` will not get called if the component implements [`static getDerivedStateFromProps`](#static-getderivedstatefromprops) or [`getSnapshotBeforeUpdate`.](#getsnapshotbeforeupdate)

- Despite its naming, `UNSAFE_componentWillReceiveProps` does not guarantee that the component *will* receive those props if your app uses modern React features like [`Suspense`.](/reference/react/Suspense) If a render attempt is suspended (for example, because the code for some child component has not loaded yet), React will throw the in-progress tree away and attempt to construct the component from scratch during the next attempt. By the time of the next render attempt, the props might be different. This is why this method is "unsafe". Code that should run only for committed updates (like resetting a subscription) should go into [`componentDidUpdate`.](#componentdidupdate)

- `UNSAFE_componentWillReceiveProps` does not mean that the component has received *different* props than the last time. You need to compare `nextProps` and `this.props` yourself to check if something changed.

- React doesn't call `UNSAFE_componentWillReceiveProps` with initial props during mounting. It only calls this method if some of component's props are going to be updated. For example, calling [`setState`](#setstate) doesn't generally trigger `UNSAFE_componentWillReceiveProps` inside the same component.

---

### `UNSAFE_componentWillUpdate(nextProps, nextState)` 

If you define `UNSAFE_componentWillUpdate`, React will call it before rendering with the new props or state. It only exists for historical reasons and should not be used in any new code. Instead, use one of the alternatives:

- If you need to run a side effect (for example, fetch data, run an animation, or reinitialize a subscription) in response to prop or state changes, move that logic to [`componentDidUpdate`](#componentdidupdate) instead.
- If you need to read some information from the DOM (for example, to save the current scroll position) so that you can use it in [`componentDidUpdate`](#componentdidupdate) later, read it inside [`getSnapshotBeforeUpdate`](#getsnapshotbeforeupdate) instead.

[See examples of migrating away from unsafe lifecycles.](https://legacy.reactjs.org/blog/2018/03/27/update-on-async-rendering.html#examples)

#### Parameters 

- `nextProps`: The next props that the component is about to render with. Compare `nextProps` to [`this.props`](#props) to determine what changed.
- `nextState`: The next state that the component is about to render with. Compare `nextState` to [`this.state`](#state) to determine what changed.

#### Returns 

`UNSAFE_componentWillUpdate` should not return anything.

#### Caveats 

- `UNSAFE_componentWillUpdate` will not get called if [`shouldComponentUpdate`](#shouldcomponentupdate) is defined and returns `false`.

- `UNSAFE_componentWillUpdate` will not get called if the component implements [`static getDerivedStateFromProps`](#static-getderivedstatefromprops) or [`getSnapshotBeforeUpdate`.](#getsnapshotbeforeupdate)

- It's not supported to call [`setState`](#setstate) (or any method that leads to `setState` being called, like dispatching a Redux action) during `componentWillUpdate`.

- Despite its naming, `UNSAFE_componentWillUpdate` does not guarantee that the component *will* update if your app uses modern React features like [`Suspense`.](/reference/react/Suspense) If a render attempt is suspended (for example, because the code for some child component has not loaded yet), React will throw the in-progress tree away and attempt to construct the component from scratch during the next attempt. By the time of the next render attempt, the props and state might be different. This is why this method is "unsafe". Code that should run only for committed updates (like resetting a subscription) should go into [`componentDidUpdate`.](#componentdidupdate)

- `UNSAFE_componentWillUpdate` does not mean that the component has received *different* props or state than the last time. You need to compare `nextProps` with `this.props` and `nextState` with `this.state` yourself to check if something changed.

- React doesn't call `UNSAFE_componentWillUpdate` with initial props and state during mounting.

---

### `static contextType` 

If you want to read [`this.context`](#context-instance-field) from your class component, you must specify which context it needs to read. The context you specify as the `static contextType` must be a value previously created by [`createContext`.](/reference/react/createContext)

```js 
class Button extends Component {
  static contextType = ThemeContext;

  render() {
    const theme = this.context;
    const className = 'button-' + theme;
    return (
      
        
      
    );
  }
}
```

---

### `static defaultProps` 

You can define `static defaultProps` to set the default props for the class. They will be used for `undefined` and missing props, but not for `null` props.

For example, here is how you define that the `color` prop should default to `'blue'`:

```js 
class Button extends Component {
  static defaultProps = ;

  render() 
}
```

If the `color` prop is not provided or is `undefined`, it will be set by default to `'blue'`:

```js
<>
  
  

---

### `static getDerivedStateFromError(error)` 

If you define `static getDerivedStateFromError`, React will call it when a child component (including distant children) throws an error during rendering. This lets you display an error message instead of clearing the UI.

Typically, it is used together with [`componentDidCatch`](#componentdidcatch) which lets you send the error report to some analytics service. A component with these methods is called an *Error Boundary*.

[See an example.](#catching-rendering-errors-with-an-error-boundary)

#### Parameters 

* `error`: The error that was thrown. In practice, it will usually be an instance of [`Error`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error) but this is not guaranteed because JavaScript allows to [`throw`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw) any value, including strings or even `null`.

#### Returns 

`static getDerivedStateFromError` should return the state telling the component to display the error message.

#### Caveats 

* `static getDerivedStateFromError` should be a pure function. If you want to perform a side effect (for example, to call an analytics service), you need to also implement [`componentDidCatch`.](#componentdidcatch)

---

### `static getDerivedStateFromProps(props, state)` 

If you define `static getDerivedStateFromProps`, React will call it right before calling [`render`,](#render) both on the initial mount and on subsequent updates. It should return an object to update the state, or `null` to update nothing.

This method exists for [rare use cases](https://legacy.reactjs.org/blog/2018/06/07/you-probably-dont-need-derived-state.html#when-to-use-derived-state) where the state depends on changes in props over time. For example, this `Form` component resets the `email` state when the `userID` prop changes:

```js 
class Form extends Component {
  state = ;

  static getDerivedStateFromProps(props, state) {
    // Any time the current user changes,
    // Reset any parts of state that are tied to that user.
    // In this simple example, that's just the email.
    if (props.userID !== state.prevUserID) {
      return ;
    }
    return null;
  }

  // ...
}
```

Note that this pattern requires you to keep a previous value of the prop (like `userID`) in state (like `prevUserID`).

#### Parameters 

- `props`: The next props that the component is about to render with.
- `state`: The next state that the component is about to render with.

#### Returns 

`static getDerivedStateFromProps` return an object to update the state, or `null` to update nothing.

#### Caveats 

- This method is fired on *every* render, regardless of the cause. This is different from [`UNSAFE_componentWillReceiveProps`](#unsafe_componentwillreceiveprops), which only fires when the parent causes a re-render and not as a result of a local `setState`.

- This method doesn't have access to the component instance. If you'd like, you can reuse some code between `static getDerivedStateFromProps` and the other class methods by extracting pure functions of the component props and state outside the class definition.

---

## Usage 

### Defining a class component 

To define a React component as a class, extend the built-in `Component` class and define a [`render` method:](#render)

```js
import  from 'react';

class Greeting extends Component {
  render() {
    return Hello, !;
  }
}
```

React will call your [`render`](#render) method whenever it needs to figure out what to display on the screen. Usually, you will return some [JSX](/learn/writing-markup-with-jsx) from it. Your `render` method should be a [pure function:](https://en.wikipedia.org/wiki/Pure_function) it should only calculate the JSX.

Similarly to [function components,](/learn/your-first-component#defining-a-component) a class component can [receive information by props](/learn/your-first-component#defining-a-component) from its parent component. However, the syntax for reading props is different. For example, if the parent component renders `

Note that Hooks (functions starting with `use`, like [`useState`](/reference/react/useState)) are not supported inside class components.

---

### Adding state to a class component 

To add [state](/learn/state-a-components-memory) to a class, assign an object to a property called [`state`](#state). To update state, call [`this.setState`](#setstate).

---

### Adding lifecycle methods to a class component 

There are a few special methods you can define on your class.

If you define the [`componentDidMount`](#componentdidmount) method, React will call it when your component is added *(mounted)* to the screen. React will call [`componentDidUpdate`](#componentdidupdate) after your component re-renders due to changed props or state. React will call [`componentWillUnmount`](#componentwillunmount) after your component has been removed *(unmounted)* from the screen.

If you implement `componentDidMount`, you usually need to implement all three lifecycles to avoid bugs. For example, if `componentDidMount` reads some state or props, you also have to implement `componentDidUpdate` to handle their changes, and `componentWillUnmount` to clean up whatever `componentDidMount` was doing.

For example, this `ChatRoom` component keeps a chat connection synchronized with props and state:

Note that in development when [Strict Mode](/reference/react/StrictMode) is on, React will call `componentDidMount`, immediately call `componentWillUnmount`, and then call `componentDidMount` again. This helps you notice if you forgot to implement `componentWillUnmount` or if its logic doesn't fully "mirror" what `componentDidMount` does.

---

### Catching rendering errors with an Error Boundary 

By default, if your application throws an error during rendering, React will remove its UI from the screen. To prevent this, you can wrap a part of your UI into an *Error Boundary*. An Error Boundary is a special component that lets you display some fallback UI instead of the part that crashed--for example, an error message.

To implement an Error Boundary component, you need to provide [`static getDerivedStateFromError`](#static-getderivedstatefromerror) which lets you update state in response to an error and display an error message to the user. You can also optionally implement [`componentDidCatch`](#componentdidcatch) to add some extra logic, for example, to log the error to an analytics service.

With [`captureOwnerStack`](/reference/react/captureOwnerStack) you can include the Owner Stack during development.

```js 
import * as React from 'react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = ;
  }

  static getDerivedStateFromError(error) {
    // Update state so the next render will show the fallback UI.
    return ;
  }

  componentDidCatch(error, info) 

  render() {
    if (this.state.hasError) 

    return this.props.children;
  }
}
```

Then you can wrap a part of your component tree with it:

```js 

```

If `Profile` or its child component throws an error, `ErrorBoundary` will "catch" that error, display a fallback UI with the error message you've provided, and send a production error report to your error reporting service.

You don't need to wrap every component into a separate Error Boundary. When you think about the [granularity of Error Boundaries,](https://www.brandondail.com/posts/fault-tolerance-react) consider where it makes sense to display an error message. For example, in a messaging app, it makes sense to place an Error Boundary around the list of conversations. It also makes sense to place one around every individual message. However, it wouldn't make sense to place a boundary around every avatar.

---

## Alternatives 

### Migrating a simple component from a class to a function 

Typically, you will [define components as functions](/learn/your-first-component#defining-a-component) instead.

For example, suppose you're converting this `Greeting` class component to a function:

Define a function called `Greeting`. This is where you will move the body of your `render` function.

```js
function Greeting() 
```

Instead of `this.props.name`, define the `name` prop [using the destructuring syntax](/learn/passing-props-to-a-component) and read it directly:

```js
function Greeting() {
  return Hello, !;
}
```

Here is a complete example:

---

### Migrating a component with state from a class to a function 

Suppose you're converting this `Counter` class component to a function:

Start by declaring a function with the necessary [state variables:](/reference/react/useState#adding-state-to-a-component)

```js 
import  from 'react';

function Counter() {
  const [name, setName] = useState('Taylor');
  const [age, setAge] = useState(42);
  // ...
```

Next, convert the event handlers:

```js 
function Counter() {
  const [name, setName] = useState('Taylor');
  const [age, setAge] = useState(42);

  function handleNameChange(e) 

  function handleAgeChange() 
  // ...
```

Finally, replace all references starting with `this` with the variables and functions you defined in your component. For example, replace `this.state.age` with `age`, and replace `this.handleNameChange` with `handleNameChange`.

Here is a fully converted component:

---

### Migrating a component with lifecycle methods from a class to a function 

Suppose you're converting this `ChatRoom` class component with lifecycle methods to a function:

First, verify that your [`componentWillUnmount`](#componentwillunmount) does the opposite of [`componentDidMount`.](#componentdidmount) In the above example, that's true: it disconnects the connection that `componentDidMount` sets up. If such logic is missing, add it first.

Next, verify that your [`componentDidUpdate`](#componentdidupdate) method handles changes to any props and state you're using in `componentDidMount`. In the above example, `componentDidMount` calls `setupConnection` which reads `this.state.serverUrl` and `this.props.roomId`. This is why `componentDidUpdate` checks whether `this.state.serverUrl` and `this.props.roomId` have changed, and resets the connection if they did. If your `componentDidUpdate` logic is missing or doesn't handle changes to all relevant props and state, fix that first.

In the above example, the logic inside the lifecycle methods connects the component to a system outside of React (a chat server). To connect a component to an external system, [describe this logic as a single Effect:](/reference/react/useEffect#connecting-to-an-external-system)

```js 
import  from 'react';

function ChatRoom() {
  const [serverUrl, setServerUrl] = useState('https://localhost:1234');

  useEffect(() => {
    const connection = createConnection(serverUrl, roomId);
    connection.connect();
    return () => ;
  }, [serverUrl, roomId]);

  // ...
}
```

This [`useEffect`](/reference/react/useEffect) call is equivalent to the logic in the lifecycle methods above. If your lifecycle methods do multiple unrelated things, [split them into multiple independent Effects.](/learn/removing-effect-dependencies#is-your-effect-doing-several-unrelated-things) Here is a complete example you can play with:

---

### Migrating a component with context from a class to a function 

In this example, the `Panel` and `Button` class components read [context](/learn/passing-data-deeply-with-context) from [`this.context`:](#context)

      
    
  );
}

export default function MyApp() 
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

When you convert them to function components, replace `this.context` with [`useContext`](/reference/react/useContext) calls:

      
    
  );
}

export default function MyApp() 
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

