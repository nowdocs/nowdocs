---
title: preinit
---

### Preiniting in an event handler 

Call `preinit` in an event handler before transitioning to a page or state where external resources will be needed. This gets the process started earlier than if you call it during the rendering of the new page or state.

```js
import  from 'react-dom';

function CallToAction() {
  const onClick = () => {
    preinit("https://example.com/wizardStyles.css", );
    startWizard();
  }
  return (
    Start Wizard
  );
}
```
