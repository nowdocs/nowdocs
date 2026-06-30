---
title: experimental_taintObjectReference
version: experimental
---

---

## Usage 

### Prevent user data from unintentionally reaching the client 

A Client Component should never accept objects that carry sensitive data. Ideally, the data fetching functions should not expose data that the current user should not have access to. Sometimes mistakes happen during refactoring. To protect against these mistakes happening down the line we can "taint" the user object in our data API.

```js
import  from 'react';

export async function getUser(id) {
  const user = await db`SELECT * FROM users WHERE id = $`;
  experimental_taintObjectReference(
    'Do not pass the entire user object to the client. ' +
      'Instead, pick off the specific properties you need for this use case.',
    user,
  );
  return user;
}
```

Now whenever anyone tries to pass this object to a Client Component, an error will be thrown with the passed in error message instead.

