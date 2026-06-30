---
title: React Developer Tools
---

## Browser extension 

The easiest way to debug websites built with React is to install the React Developer Tools browser extension. It is available for several popular browsers:

* [Install for **Chrome**](https://chrome.google.com/webstore/detail/react-developer-tools/fmkadmapgofadopljbjfkapdkoienihi?hl=en)
* [Install for **Firefox**](https://addons.mozilla.org/en-US/firefox/addon/react-devtools/)
* [Install for **Edge**](https://microsoftedge.microsoft.com/addons/detail/react-developer-tools/gpphkfbcpidddadnkolkpfckpihlkkil)

Now, if you visit a website **built with React,** you will see the _Components_ and _Profiler_ panels.

### Safari and other browsers 
For other browsers (for example, Safari), install the [`react-devtools`](https://www.npmjs.com/package/react-devtools) npm package:
```bash
# Yarn
yarn global add react-devtools

# Npm
npm install -g react-devtools
```

Next open the developer tools from the terminal:
```bash
react-devtools
```

Then connect your website by adding the following `` tag to the beginning of your website's ``:
```html 

  
    
```

Reload your website in the browser now to view it in developer tools.

## Mobile (React Native) 

To inspect apps built with [React Native](https://reactnative.dev/), you can use [React Native DevTools](https://reactnative.dev/docs/react-native-devtools), the built-in debugger that deeply integrates React Developer Tools. All features work identically to the browser extension, including native element highlighting and selection.

[Learn more about debugging in React Native.](https://reactnative.dev/docs/debugging)

> For versions of React Native earlier than 0.76, please use the standalone build of React DevTools by following the [Safari and other browsers](#safari-and-other-browsers) guide above.
