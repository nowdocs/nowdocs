---
title: react-dom/test-utils Deprecation Warnings
---

## ReactDOMTestUtils.act() warning 

`act` from `react-dom/test-utils` has been deprecated in favor of `act` from `react`.

Before:

```js
import  from 'react-dom/test-utils';
```

After:

```js
import  from 'react';
```

## Rest of ReactDOMTestUtils APIS 

All APIs except `act` have been removed.

The React Team recommends migrating your tests to [@testing-library/react](https://testing-library.com/docs/react-testing-library/intro/) for a modern and well supported testing experience.

### ReactDOMTestUtils.renderIntoDocument 

`renderIntoDocument` can be replaced with `render` from `@testing-library/react`.

Before:

```js
import  from 'react-dom/test-utils';

renderIntoDocument();
```

After:

```js
import  from '@testing-library/react';

render();
```

### ReactDOMTestUtils.Simulate 

`Simulate` can be replaced with `fireEvent` from `@testing-library/react`.

Before:

```js
import  from 'react-dom/test-utils';

const element = document.querySelector('button');
Simulate.click(element);
```

After:

```js
import  from '@testing-library/react';

const element = document.querySelector('button');
fireEvent.click(element);
```

Be aware that `fireEvent` dispatches an actual event on the element and doesn't just synthetically call the event handler.

### List of all removed APIs 

- `mockComponent()`
- `isElement()`
- `isElementOfType()`
- `isDOMComponent()`
- `isCompositeComponent()`
- `isCompositeComponentWithType()`
- `findAllInRenderedTree()`
- `scryRenderedDOMComponentsWithClass()`
- `findRenderedDOMComponentWithClass()`
- `scryRenderedDOMComponentsWithTag()`
- `findRenderedDOMComponentWithTag()`
- `scryRenderedComponentsWithType()`
- `findRenderedComponentWithType()`
- `renderIntoDocument`
- `Simulate`
