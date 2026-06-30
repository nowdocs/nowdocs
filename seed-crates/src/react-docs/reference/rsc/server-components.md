---
title: Server Components
---

This separate environment is the "server" in React Server Components. Server Components can run once at build time on your CI server, or they can be run for each request using a web server.

### Server Components without a Server 
Server components can run at build time to read from the filesystem or fetch static content, so a web server is not required. For example, you may want to read static data from a content management system.

Without Server Components, it's common to fetch static data on the client with an Effect:
```js
// bundle.js
import marked from 'marked'; // 35.9K (11.2K gzipped)
import sanitizeHtml from 'sanitize-html'; // 206K (63.3K gzipped)

function Page() {
  const [content, setContent] = useState('');
  // NOTE: loads *after* first page render.
  useEffect(() => {
    fetch(`/api/content/$`).then((data) => );
  }, [page]);

  return ;
}
```
```js
// api.js
app.get(`/api/content/:page`, async (req, res) => {
  const page = req.params.page;
  const content = await file.readFile(`$.md`);
  res.send();
});
```

This pattern means users need to download and parse an additional 75K (gzipped) of libraries, and wait for a second request to fetch the data after the page loads, just to render static content that will not change for the lifetime of the page.

With Server Components, you can render these components once at build time:

```js
import marked from 'marked'; // Not included in bundle
import sanitizeHtml from 'sanitize-html'; // Not included in bundle

async function Page() {
  // NOTE: loads *during* render, when the app is built.
  const content = await file.readFile(`$.md`);

  return ;
}
```

The rendered output can then be server-side rendered (SSR) to HTML and uploaded to a CDN. When the app loads, the client will not see the original `Page` component, or the expensive libraries for rendering the markdown. The client will only see the rendered output:

```js

```

This means the content is visible during first page load, and the bundle does not include the expensive libraries needed to render the static content.

### Server Components with a Server 
Server Components can also run on a web server during a request for a page, letting you access your data layer without having to build an API. They are rendered before your application is bundled, and can pass data and JSX as props to Client Components.

Without Server Components, it's common to fetch dynamic data on the client in an Effect:

```js
// bundle.js
function Note() {
  const [note, setNote] = useState('');
  // NOTE: loads *after* first render.
  useEffect(() => {
    fetch(`/api/notes/$`).then(data => );
  }, [id]);

  return (
    
      

In the following example, the `Notes` Server Component imports an `Expandable` Client Component that uses state to toggle its `expanded` state:
```js
// Server Component
import Expandable from './Expandable';

async function Notes() {
  const notes = await db.notes.getAll();
  return (
    
      
    
  )
}
```
```js
// Client Component
"use client"

export default function Expandable() 
      >
        Toggle
      
      
    
  )
}
```

This works by first rendering `Notes` as a Server Component, and then instructing the bundler to create a bundle for the Client Component `Expandable`. In the browser, the Client Components will see output of the Server Components passed as props:

```js

  
  

  
    
    
    
  

```

### Async components with Server Components 

Server Components introduce a new way to write Components using async/await. When you `await` in an async component, React will suspend and wait for the promise to resolve before resuming rendering. This works across server/client boundaries with streaming support for Suspense.

You can even create a promise on the server, and await it on the client:

```js
// Server Component
import db from './database';

async function Page() {
  // Will suspend the Server Component.
  const note = await db.notes.get(id);

  // NOTE: not awaited, will start here and await on the client.
  const commentsPromise = db.comments.get(note.id);
  return (
    
      
      
    
  );
}
```

```js
// Client Component
"use client";
import  from 'react';

function Comments() {
  // NOTE: this will resume the promise from the server.
  // It will suspend until the data is available.
  const comments = use(commentsPromise);
  return comments.map(comment => );
}
```

The `note` content is important data for the page to render, so we `await` it on the server. The comments are below the fold and lower-priority, so we start the promise on the server, and wait for it on the client with the `use` API. This will Suspend on the client, without blocking the `note` content from rendering.

Since async components are not supported on the client, we await the promise with `use`.
