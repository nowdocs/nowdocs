---
title: Form Component
description: Learn how to use the ` **client-side navigation** on submission, and **progressive enhancement**.

It's useful for forms that update URL search params as it reduces the boilerplate code needed to achieve the above.

Basic usage:

  )
}
```

```jsx filename="/app/ui/search.js" switcher
import Form from 'next/form'

export default function Search() 
```

  )
}
```

```jsx filename="/ui/search.js" switcher
import Form from 'next/form'

export default function Search() 
```

## Reference

The behavior of the `

### `action` (string) Props

### Caveats

- **`onSubmit`**: Can be used to handle form submission logic. However, calling `event.preventDefault()` will override `
  )
}
```

```jsx filename="/app/page.js" switcher
import Form from 'next/form'

export default function Page() 
```

When the user updates the query input field and submits the form, the form data will be encoded into the URL as search params, e.g. `/search?query=abc`.

> **Good to know**: If you pass an empty string `""` to `action`, the form will navigate to the same route with updated search params.

On the results page, you can access the query using the [`searchParams`](/docs/app/api-reference/file-conventions/page#searchparams-optional) `page.js` prop and use it to fetch data from an external source.

```tsx filename="/app/search/page.tsx" switcher
import  from '@/lib/search'

export default async function SearchPage(: {
  searchParams: Promise<>
}) 
```

```jsx filename="/app/search/page.js" switcher
import  from '@/lib/search'

export default async function SearchPage() 
```

When the `
  )
}
```

```jsx filename="/app/ui/search-button.js" switcher
import Form from 'next/form'
import  from '@/ui/search-button'

export default function Page() 
```

### Mutations with Server Actions

You can perform mutations by passing a function to the `action` prop.

```tsx filename="/app/posts/create/page.tsx" switcher
import Form from 'next/form'
import  from '@/posts/actions'

export default function Page() 
```

```jsx filename="/app/posts/create/page.js" switcher
import Form from 'next/form'
import  from '@/posts/actions'

export default function Page() 
```

After a mutation, it's common to redirect to the new resource. You can use the [`redirect`](/docs/app/guides/redirecting) function from `next/navigation` to navigate to the new post page.

> **Good to know**: Since the "destination" of the form submission is not known until the action is executed, `
