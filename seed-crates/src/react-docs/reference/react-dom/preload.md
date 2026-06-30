---
title: preload
---

### Preloading in an event handler 

Call `preload` in an event handler before transitioning to a page or state where external resources will be needed. This gets the process started earlier than if you call it during the rendering of the new page or state.

```js
import  from 'react-dom';

function CallToAction() {
  const onClick = () => {
    preload("https://example.com/wizardStyles.css", );
    startWizard();
  }
  return (
    Start Wizard
  );
}
```
