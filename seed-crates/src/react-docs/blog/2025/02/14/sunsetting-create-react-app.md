---
title: "Sunsetting Create React App"
author: Matt Carroll and Ricky Hanlon
date: 2025/02/14
description: Today, we’re deprecating Create React App for new apps, and encouraging existing apps to migrate to a framework, or to migrate to a build tool like Vite, Parcel, or RSBuild. We’re also providing docs for when a framework isn’t a good fit for your project, you want to build your own framework, or you just want to learn how React works by building a React app from scratch.
---

February 14, 2025 by [Matt Carroll](https://twitter.com/mattcarrollcode) and [Ricky Hanlon](https://bsky.app/profile/ricky.fm)

---

-----

When we released Create React App in 2016, there was no clear way to build a new React app.

To create a React app, you had to install a bunch of tools and wire them up together yourself to support basic features like JSX, linting, and hot reloading. This was very tricky to do correctly, so the [community](https://github.com/react-boilerplate/react-boilerplate) [created](https://github.com/kriasoft/react-starter-kit) [boilerplates](https://github.com/petehunt/react-boilerplate) for [common](https://github.com/gaearon/react-hot-boilerplate) [setups](https://github.com/erikras/react-redux-universal-hot-example). However, boilerplates were difficult to update and fragmentation made it difficult for React to release new features.

Create React App solved these problems by combining several tools into a single recommended configuration. This allowed apps a simple way to upgrade to new tooling features, and allowed the React team to deploy non-trivial tooling changes (Fast Refresh support, React Hooks lint rules) to the broadest possible audience.

This model became so popular that there's an entire category of tools working this way today.

## Deprecating Create React App 

Although Create React App makes it easy to get started, [there are several limitations](#limitations-of-build-tools) that make it difficult to build high performant production apps. In principle, we could solve these problems by essentially evolving it into a [framework](#why-we-recommend-frameworks).

However, since Create React App currently has no active maintainers, and there are many existing frameworks that solve these problems already, we’ve decided to deprecate Create React App.

Starting today, if you install a new app, you will see a deprecation warning:

We've also added a deprecation notice to the Create React App [website](https://create-react-app.dev/) and GitHub [repo](https://github.com/facebook/create-react-app). Create React App will continue working in maintenance mode, and we've published a new version of Create React App to work with React 19.

## How to Migrate to a Framework 
We recommend [creating new React apps](/learn/creating-a-react-app) with a framework. All the frameworks we recommend support client-side rendering ([CSR](https://developer.mozilla.org/en-US/docs/Glossary/CSR)) and single-page apps ([SPA](https://developer.mozilla.org/en-US/docs/Glossary/SPA)), and can be deployed to a CDN or static hosting service without a server.

For existing apps, these guides will help you migrate to a client-only SPA:

* [Next.js’ Create React App migration guide](https://nextjs.org/docs/app/building-your-application/upgrading/from-create-react-app)
* [React Router’s framework adoption guide](https://reactrouter.com/upgrading/component-routes).
* [Expo webpack to Expo Router migration guide](https://docs.expo.dev/router/migrate/from-expo-webpack/)

## How to Migrate to a Build Tool 

If your app has unusual constraints, or you prefer to solve these problems by building your own framework, or you just want to learn how react works from scratch, you can roll your own custom setup with React using Vite, Parcel or Rsbuild.

For existing apps, these guides will help you migrate to a build tool:

* [Vite Create React App migration guide](https://www.robinwieruch.de/vite-create-react-app/)
* [Parcel Create React App migration guide](https://parceljs.org/migration/cra/)
* [Rsbuild Create React App migration guide](https://rsbuild.dev/guide/migration/cra)

To help get started with Vite, Parcel or Rsbuild, we've added new docs for [Building a React App from Scratch](/learn/build-a-react-app-from-scratch).

Continue reading to learn more about the [limitations of build tools](#limitations-of-build-tools) and [why we recommend frameworks](#why-we-recommend-frameworks).

## Limitations of Build Tools 

Create React App and build tools like it make it easy to get started building a React app. After running `npx create-react-app my-app`, you get a fully configured React app with a development server, linting, and a production build.

For example, if you're building an internal admin tool, you can start with a landing page:

```js
export default function App() 
```

This allows you to immediately start coding in React with features like JSX, default linting rules, and a bundler to run in both development and production. However, this setup is missing the tools you need to build a real production app.

Most production apps need solutions to problems like routing, data fetching, and code splitting.

### Routing 

Create React App does not include a specific routing solution. If you're just getting started, one option is to use `useState` to switch between routes. But doing this means that you can't share links to your app - every link would go to the same page - and structuring your app becomes difficult over time:

```js
import  from 'react';

import Home from './Home';
import Dashboard from './Dashboard';

export default function App() {
  // ❌ Routing in state does not create URLs
  const [route, setRoute] = useState('home');
  return (
    
      {route === 'home' && 

---

_Thank you to [Dan Abramov](https://bsky.app/profile/danabra.mov) for creating Create React App, and [Joe Haddad](https://github.com/Timer), [Ian Schmitz](https://github.com/ianschmitz), [Brody McKee](https://github.com/mrmckeb), and [many others](https://github.com/facebook/create-react-app/graphs/contributors) for maintaining Create React App over the years. Thank you to [Brooks Lybrand](https://bsky.app/profile/brookslybrand.bsky.social), [Dan Abramov](https://bsky.app/profile/danabra.mov), [Devon Govett](https://bsky.app/profile/devongovett.bsky.social), [Eli White](https://x.com/Eli_White), [Jack Herrington](https://bsky.app/profile/jherr.dev), [Joe Savona](https://x.com/en_JS), [Lauren Tan](https://bsky.app/profile/no.lol), [Lee Robinson](https://x.com/leeerob), [Mark Erikson](https://bsky.app/profile/acemarke.dev), [Ryan Florence](https://x.com/ryanflorence), [Sophie Alpert](https://bsky.app/profile/sophiebits.com), [Tanner Linsley](https://bsky.app/profile/tannerlinsley.com), and [Theo Browne](https://x.com/theo) for reviewing and providing feedback on this post._

