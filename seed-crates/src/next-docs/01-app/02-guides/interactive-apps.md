---
title: Building interactive apps
description: Learn how to build responsive interactions with Server Functions, transitions, optimistic UI, and pending feedback.
nav_title: Interactive apps
related:
  description: Learn more about the APIs used in this guide.
  links:
    - app/api-reference/functions/refresh
    - app/api-reference/directives/use-cache
    - app/api-reference/functions/cacheTag
    - app/api-reference/functions/updateTag
---

When a user interaction requires server-side work, the result is not available immediately. Network requests take an unknown amount of time to complete and may succeed or fail. While waiting, the client should provide feedback and respond to those states. Otherwise, users are left looking at stale data and may wonder whether anything is happening.

This guide adds responsive feedback to a task management app called _Taskboard_. Each step handles a different kind of pending work, from streaming slow reads to confirming mutations before the server responds.

> **Good to know:** The patterns in this guide also improve [Core Web Vitals](https://web.dev/articles/vitals). Streaming with [`
          </>
        ))}
      
    
  )
}
```

Each section component awaits its own data:

```tsx filename="features/task/components/task-detail.tsx"
export async function TaskDetail() 
```

With this version, the submit handler runs, but two separate `useState` calls manage the pending state. The dialog closes before the board shows the new task, because `setIsOpen(false)` fires outside of a transition. The form fields also keep their values across submissions unless you track a reset key yourself.

`useActionState` handles all three concerns: `isPending` for the button, key-based reset for the fields, and a wrap around the dialog close:

```tsx filename="features/task/components/create-task-modal.tsx"
'use client'

import  from 'react'
import  from '@/features/task/task-actions'

export function CreateTaskModal() {
  const [isOpen, setIsOpen] = useState(false)

  const [, formAction, isPending] = useActionState(
    async (prev, formData) => {
      const title = String(formData.get('title'))
      if (!title.trim()) return prev

      await createTask()

      startTransition(() => setIsOpen(false))
      return 
    },
    
  )

  return (
    
  )
}
```

The `key` in the returned state controls form reset. On success, `key` increments, which remounts the `` and resets every input inside it.

`useActionState` runs the action as a transition, so the dialog stays open and `isPending` stays `true` while `createTask` runs. State updates after the `await` aren't automatically part of that transition, so wrapping `setIsOpen(false)` in `startTransition` runs the close as part of the transition.

With that wrap in place, React batches the dialog close with the board update that `refresh()` triggers inside `createTask`, so the new task appears at the moment the dialog closes. Without it, the dialog closes first and the board updates a frame later.

> **Good to know:** This limitation is documented in React under [React doesn't treat my state update after `await` as a transition](https://react.dev/reference/react/useTransition#react-doesnt-treat-my-state-update-after-await-as-a-transition). Until it's fixed, wrapping post-`await` state updates in `startTransition` is the recommended workaround.

The same post-`await` window is where side effects like toasts belong:

```tsx
await createTask()

startTransition(() => setIsOpen(false))
toast.success('Task created')
```

Side effects that don't affect rendered state, such as analytics, toasts, and focus changes, run after the `await` resolves. They don't need a transition because they don't update React state.

The button text now changes to "Creating..." and dims on submit. When the server responds, the fields clear, the dialog closes, the new task appears on the board, and a toast confirms the action.

> **Good to know:** The companion app adds status, priority, assignee, and label pickers to the modal. The pattern is the same: hidden inputs track each picker's state, and `key` resets them all on success.

### Step 7: Signal pending deletion to a parent

Each comment card has a delete button that removes the comment on the server, and the comment disappears from the list after the next render.

Between the click and that render, the card still shows the old data. The list itself can't show the pending removal, so the signal has to come from the button.

This step reuses the `data-pending` CSS hook from Step 3, but drives it from `useOptimistic(false)` instead of the `isPending` from `useTransition`.

Here the Server Function is called directly:

```tsx filename="features/task/components/delete-button.tsx"
'use client'

import  from 'lucide-react'
import  from '@/features/task/task-actions'

export function DeleteButton() 
      aria-label="Delete comment"
    >
      
```

A prefetch only pays off when the destination's reads are cached: the tags from the previous subsections give the prefetched output a stable slot in the [Client Cache](/docs/app/glossary#client-cache), so a later navigation reuses it instead of refetching. In the companion app, each task card is a [``](/docs/app/api-reference/components/link) to the task detail page. The same element is also `draggable`, and the browser distinguishes a click from a drag natively. Because the cards are links, the detail page prefetches as they scroll into view, so the destination paints instantly by the time the user clicks. See the [Prefetching guide](/docs/app/guides/prefetching) and [Runtime Prefetching guide](/docs/app/guides/runtime-prefetching) for prefetch modes, and the [Instant Navigation guide](/docs/app/guides/instant-navigation) for validating that navigations stay instant in production.

## Next steps

The patterns in this guide combine a handful of primitives:

| Situation                                                                 | Use                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Slow data should stream in without blocking the page                      | [``](https://react.dev/reference/react/Suspense)                                                                                                                                                                                                                   |
| A value should update while async work runs                               | [`useOptimistic`](https://react.dev/reference/react/useOptimistic)                                                                                                                                                                                                           |
| Async work needs pending state, error handling, or coordinated UI updates | [`useTransition`](https://react.dev/reference/react/useTransition)                                                                                                                                                                                                           |
| A form needs pending, reset, and result state                             | [`useActionState`](https://react.dev/reference/react/useActionState)                                                                                                                                                                                                         |
| An ancestor should show pending state for work happening elsewhere        | [`data-pending`](https://react.dev/reference/react-dom/components/form#props) attribute styled with CSS                                                                                                                                                                      |
| Reusable reads should survive across requests and stay fresh after writes | [`'use cache'`](/docs/app/api-reference/directives/use-cache) with [`cacheTag`](/docs/app/api-reference/functions/cacheTag), revalidated by [`updateTag`](/docs/app/api-reference/functions/updateTag) or [`revalidateTag`](/docs/app/api-reference/functions/revalidateTag) |
| Navigation between pages of an interactive app should feel instant        | [``](/docs/app/api-reference/components/link) prefetching with [Partial Prefetching](/docs/app/guides/adopting-partial-prefetching), and [Runtime prefetching](/docs/app/guides/runtime-prefetching) for request-dependent reads                                       |

Most patterns mix two or more of these. Reach for whichever primitive fits the constraint you're solving.

Related guides:

- [Streaming](/docs/app/guides/streaming) for loading boundaries and `` patterns
- [Instant Navigation](/docs/app/guides/instant-navigation) for validating that navigations stay instant
- [View Transitions](/docs/app/guides/view-transitions) for animating state changes
