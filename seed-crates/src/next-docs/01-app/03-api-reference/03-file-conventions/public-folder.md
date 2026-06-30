---
title: public Folder
nav_title: public
description: Next.js allows you to serve static files, like images, in the public directory. You can learn how it works here.
---

Next.js can serve static files, like images, under a folder called `public` in the root directory. Files inside `public` can then be referenced by your code starting from the base URL (`/`).

For example, the file `public/avatars/me.png` can be viewed by visiting the `/avatars/me.png` path. The code to display that image might look like:

```jsx filename="avatar.js"
import Image from 'next/image'

export function Avatar() {
  return 

