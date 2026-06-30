---
title: "Introducing react.dev"
author: Dan Abramov and Rachel Nabors
date: 2023/03/16
description: Today we are thrilled to launch react.dev, the new home for React and its documentation. In this post, we would like to give you a tour of the new site.
---

March 16, 2023 by [Dan Abramov](https://bsky.app/profile/danabra.mov) and [Rachel Nabors](https://twitter.com/rachelnabors)

---

---

## tl;dr 

* The new React site ([react.dev](https://react.dev)) teaches modern React with function components and Hooks.
* We've included diagrams, illustrations, challenges, and over 600 new interactive examples.
* The previous React documentation site has now moved to [legacy.reactjs.org](https://legacy.reactjs.org).

## New site, new domain, new homepage 

First, a little bit of housekeeping.

To celebrate the launch of the new docs and, more importantly, to clearly separate the old and the new content, we've moved to the shorter [react.dev](https://react.dev) domain. The old [reactjs.org](https://reactjs.org) domain will now redirect here.

The old React docs are now archived at [legacy.reactjs.org](https://legacy.reactjs.org). All existing links to the old content will automatically redirect there to avoid "breaking the web", but the legacy site will not get many more updates.

Believe it or not, React will soon be ten years old. In JavaScript years, it's like a whole century! We've [refreshed the React homepage](https://react.dev) to reflect why we think React is a great way to create user interfaces today, and updated the getting started guides to more prominently mention modern React-based frameworks.

If you haven't seen the new homepage yet, check it out!

## Going all-in on modern React with Hooks 

When we released React Hooks in 2018, the Hooks docs assumed the reader is familiar with class components. This helped the community adopt Hooks very swiftly, but after a while the old docs failed to serve the new readers. New readers had to learn React twice: once with class components and then once again with Hooks.

**The new docs teach React with Hooks from the beginning.** The docs are divided in two main sections:

* **[Learn React](/learn)** is a self-paced course that teaches React from scratch.
* **[API Reference](/reference)** provides the details and usage examples for every React API.

Let's have a closer look at what you can find in each section.

## Quick start 

The Learn section begins with the [Quick Start](/learn) page. It is a short introductory tour of React. It introduces the syntax for concepts like components, props, and state, but doesn't go into much detail on how to use them.

If you like to learn by doing, we recommend checking out the [Tic-Tac-Toe Tutorial](/learn/tutorial-tic-tac-toe) next. It walks you through building a little game with React, while teaching the skills you'll use every day. Here's what you'll build:

We'd also like to highlight [Thinking in React](/learn/thinking-in-react)—that's the tutorial that made React "click" for many of us. **We've updated both of these classic tutorials to use function components and Hooks,** so they're as good as new.

## Learn React step by step 

We'd like everyone in the world to have an equal opportunity to learn React for free on their own.

This is why the Learn section is organized like a self-paced course split into chapters. The first two chapters describe the fundamentals of React. If you're new to React, or want to refresh it in your memory, start here:

- **[Describing the UI](/learn/describing-the-ui)** teaches how to display information with components.
- **[Adding Interactivity](/learn/adding-interactivity)** teaches how to update the screen in response to user input.

The next two chapters are more advanced, and will give you a deeper insight into the trickier parts:

- **[Managing State](/learn/managing-state)** teaches how to organize your logic as your app grows in complexity.
- **[Escape Hatches](/learn/escape-hatches)** teaches how you can "step outside" React, and when it makes most sense to do so.

Every chapter consists of several related pages. Most of these pages teach a specific skill or a technique—for example, [Writing Markup with JSX](/learn/writing-markup-with-jsx), [Updating Objects in State](/learn/updating-objects-in-state), or [Sharing State Between Components](/learn/sharing-state-between-components). Some of the pages focus on explaining an idea—like [Render and Commit](/learn/render-and-commit), or [State as a Snapshot](/learn/state-as-a-snapshot). And there are a few, like [You Might Not Need an Effect](/learn/you-might-not-need-an-effect), that share our suggestions based on what we've learned over these years.

You don't have to read these chapters as a sequence. Who has the time for this?! But you could. Pages in the Learn section only rely on concepts introduced by the earlier pages. If you want to read it like a book, go for it!

### Check your understanding with challenges 

Most pages in the Learn section end with a few challenges to check your understanding. For example, here are a few challenges from the page about [Conditional Rendering](/learn/conditional-rendering#challenges).

You don't have to solve them right now! Unless you *really* want to.

#### Show the item importance with `&&` 

In this example, each `Item` receives a numerical `importance` prop. Use the `&&` operator to render "_(Importance: X)_" in italics, but only for items that have non-zero importance. Your item list should end up looking like this:

* Space suit _(Importance: 9)_
* Helmet with a golden leaf
* Photo of Tam _(Importance: 6)_

Don't forget to add a space between the two labels!

Note that you must write `importance > 0 && ...` rather than `importance && ...` so that if the `importance` is `0`, `0` isn't rendered as the result!

In this solution, two separate conditions are used to insert a space between then name and the importance label. Alternatively, you could use a Fragment with a leading space: `importance > 0 && <> ...</>` or add a space immediately inside the ``:  `importance > 0 &&  ...`.

Notice the "Show solution" button in the left bottom corner. It's handy if you want to check yourself!

### Build an intuition with diagrams and illustrations 

When we couldn't figure out how to explain something with code and words alone, we've added diagrams that help provide some intuition. For example, here is one of the diagrams from [Preserving and Resetting State](/learn/preserving-and-resetting-state):

You'll also see some illustrations throughout the docs--here's one of the [browser painting the screen](/learn/render-and-commit#epilogue-browser-paint):

Some API pages also include [Troubleshooting](/reference/react/useEffect#troubleshooting) (for common problems) and [Alternatives](/reference/react-dom/findDOMNode#alternatives) (for deprecated APIs).

We hope that this approach will make the API reference useful not only as a way to look up an argument, but as a way to see all the different things you can do with any given API—and how it connects to the other ones.

## What's next? 

That's a wrap for our little tour! Have a look around the new website, see what you like or don't like, and keep the feedback coming in our [issue tracker](https://github.com/reactjs/react.dev/issues).

We acknowledge this project has taken a long time to ship. We wanted to maintain a high quality bar that the React community deserves. While writing these docs and creating all of the examples, we found mistakes in some of our own explanations, bugs in React, and even gaps in the React design that we are now working to address. We hope that the new documentation will help us hold React itself to a higher bar in the future.

We've heard many of your requests to expand the content and functionality of the website, for example:

- Providing a TypeScript version for all examples;
- Creating the updated performance, testing, and accessibility guides;
- Documenting React Server Components independently from the frameworks that support them;
- Working with our international community to get the new docs translated;
- Adding missing features to the new website (for example, RSS for this blog).

Now that [react.dev](https://react.dev/) is out, we will be able to shift our focus from "catching up" with the third-party React educational resources to adding new information and further improving our new website.

We think there's never been a better time to learn React.

## Who worked on this? 

On the React team, [Rachel Nabors](https://twitter.com/rachelnabors/) led the project (and provided the illustrations), and [Dan Abramov](https://bsky.app/profile/danabra.mov) designed the curriculum. They co-authored most of the content together as well.

Of course, no project this large happens in isolation. We have a lot of people to thank!

[Sylwia Vargas](https://twitter.com/SylwiaVargas) overhauled our examples to go beyond "foo/bar/baz" and kittens, and feature scientists, artists and cities from around the world. [Maggie Appleton](https://twitter.com/Mappletons) turned our doodles into a clear diagram system.

Thanks to [David McCabe](https://twitter.com/mcc_abe), [Sophie Alpert](https://twitter.com/sophiebits), [Rick Hanlon](https://twitter.com/rickhanlonii), [Andrew Clark](https://twitter.com/acdlite), and [Matt Carroll](https://twitter.com/mattcarrollcode) for additional writing contributions. We'd also like to thank [Natalia Tepluhina](https://twitter.com/n_tepluhina) and [Sebastian Markbåge](https://twitter.com/sebmarkbage) for their ideas and feedback.

Thanks to [Dan Lebowitz](https://twitter.com/lebo) for the site design and [Razvan Gradinar](https://dribbble.com/GradinarRazvan) for the sandbox design.

On the development front, thanks to [Jared Palmer](https://twitter.com/jaredpalmer) for prototype development. Thanks to [Dane Grant](https://twitter.com/danecando) and [Dustin Goodman](https://twitter.com/dustinsgoodman) from [ThisDotLabs](https://www.thisdot.co/) for their support on UI development. Thanks to [Ives van Hoorne](https://twitter.com/CompuIves), [Alex Moldovan](https://twitter.com/alexnmoldovan), [Jasper De Moor](https://twitter.com/JasperDeMoor), and [Danilo Woznica](https://twitter.com/danilowoz) from [CodeSandbox](https://codesandbox.io/) for their work with sandbox integration. Thanks to [Rick Hanlon](https://twitter.com/rickhanlonii) for spot development and design work, finessing our colors and finer details. Thanks to [Harish Kumar](https://www.strek.in/) and [Luna Ruan](https://twitter.com/lunaruan) for adding new features to the site and helping maintain it.

Huge thanks to the folks who volunteered their time to participate in the alpha and beta testing program. Your enthusiasm and invaluable feedback helped us shape these docs. A special shout out to our beta tester, [Debbie O'Brien](https://twitter.com/debs_obrien), who gave a talk about her experience using the React docs at React Conf 2021.

Finally, thanks to the React community for being the inspiration behind this effort. You are the reason we do this, and we hope that the new docs will help you use React to build any user interface that you want.
