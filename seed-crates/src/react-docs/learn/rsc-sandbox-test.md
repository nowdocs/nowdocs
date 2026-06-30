---
title: RSC Sandbox Test
---

## Basic Server Component 

## Server + Client Components 

## Async Server Component with Suspense 

    
  );
}
```

```js src/Albums.js
async function fetchAlbums() 

export default async function Albums() {
  const albums = await fetchAlbums();
  return (
    
      {albums.map(album => (
        
      ))}
    
  );
}
```

## Streaming Proof 

This demo proves streaming is incremental. The shell renders instantly with a `
    
  );
}
```

```js src/SlowData.js
import Timestamp from './Timestamp';

async function fetchData() 

export default async function SlowData() {
  const items = await fetchData();
  return (
    
      Data streamed in at: 

## Flight Data Types 

This demo passes Map, Set, Date, and BigInt from a server component through the Flight stream to a client component, proving the full Flight protocol type system works end-to-end.

## Promise Streaming with use() 

The server creates a promise (resolves in 2s) and passes it as a prop through a parent async component that suspends for 3s. When the parent reveals at ~3s, the promise is already resolved — so `use()` returns instantly with no inner fallback. The elapsed time should be ~3000ms (the parent's delay), not ~5000ms (which would mean the promise restarted on the client).

        
      
    
  );
}
```

```js src/SlowParent.js
export default async function SlowParent() {
  await new Promise(resolve => setTimeout(resolve, 3000));
  return ;
}
```

```js src/UserCard.js
'use client';
import  from 'react';

function now() 
export default function UserCard() {
  const user = use(userPromise);
  const elapsed = now() - serverTime;
  return (
    
      
      
      
        Rendered ms after server created the promise.
      
      
        ~3000ms = promise already resolved, waited only for parent.
        ~5000ms would mean the promise restarted on the client.
      
    
  );
}
```

## Flight Data Types in Server Actions 

This demo sends Map, Set, Date, and BigInt from a client component *to* a server action via `encodeReply`/`decodeReply`, then verifies the types survived the round trip.

## Server Action Mutation + Re-render 

The server action mutates server-side data and returns a confirmation string. The updated list is only visible because the framework automatically re-renders the entire server component tree after the action completes — the server component re-reads the data and streams the new UI to the client.

## Inline Server Actions 

Server actions defined inline inside a server component with `'use server'` on the function body. The action closes over module-level state and is passed as a prop — no separate `actions.js` file needed.

## Server Functions 

