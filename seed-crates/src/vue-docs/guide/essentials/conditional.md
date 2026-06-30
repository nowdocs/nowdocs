# Conditional Rendering 

  

  

import  from 'vue'
const awesome = ref(true)

## `v-if` 

The directive `v-if` is used to conditionally render a block. The block will only be rendered if the directive's expression returns a truthy value.

```vue-html
Vue is awesome!
```

## `v-else` 

You can use the `v-else` directive to indicate an "else block" for `v-if`:

```vue-html
Toggle

Vue is awesome!
Oh no 😢
```

  Toggle
  Vue is awesome!
  Oh no 😢

[Try it in the Playground](https://play.vuejs.org/#eNpFjkEOgjAQRa8ydIMulLA1hegJ3LnqBskAjdA27RQXhHu4M/GEHsEiKLv5mfdf/sBOxux7j+zAuCutNAQOyZtcKNkZbQkGsFjBCJXVHcQBjYUSqtTKERR3dLpDyCZmQ9bjViiezKKgCIGwM21BGBIAv3oireBYtrK8ZYKtgmg5BctJ13WLPJnhr0YQb1Lod7JaS4G8eATpfjMinjTphC8wtg7zcwNKw/v5eC1fnvwnsfEDwaha7w==)

[Try it in the Playground](https://play.vuejs.org/#eNpFjj0OwjAMha9iMsEAFWuVVnACNqYsoXV/RJpEqVOQqt6DDYkTcgRSWoplWX7y56fXs6O1u84jixlvM1dbSoXGuzWOIMdCekXQCw2QS5LrzbQLckje6VEJglDyhq1pMAZyHidkGG9hhObRYh0EYWOVJAwKgF88kdFwyFSdXRPBZidIYDWvgqVkylIhjyb4ayOIV3votnXxfwrk2SPU7S/PikfVfsRnGFWL6akCbeD9fLzmK4+WSGz4AA5dYQY=)

A `v-else` element must immediately follow a `v-if` or a `v-else-if` element - otherwise it will not be recognized.

## `v-else-if` 

The `v-else-if`, as the name suggests, serves as an "else if block" for `v-if`. It can also be chained multiple times:

```vue-html

  A

  B

  C

  Not A/B/C

```

Similar to `v-else`, a `v-else-if` element must immediately follow a `v-if` or a `v-else-if` element.

## `v-if` on `` 

Because `v-if` is a directive, it has to be attached to a single element. But what if we want to toggle more than one element? In this case we can use `v-if` on a `` element, which serves as an invisible wrapper. The final rendered result will not include the `` element.

```vue-html

  Title
  Paragraph 1
  Paragraph 2

```

`v-else` and `v-else-if` can also be used on ``.

## `v-show` 

Another option for conditionally displaying an element is the `v-show` directive. The usage is largely the same:

```vue-html
Hello!
```

The difference is that an element with `v-show` will always be rendered and remain in the DOM; `v-show` only toggles the `display` CSS property of the element.

`v-show` doesn't support the `` element, nor does it work with `v-else`.

## `v-if` vs. `v-show` 

`v-if` is "real" conditional rendering because it ensures that event listeners and child components inside the conditional block are properly destroyed and re-created during toggles.

`v-if` is also **lazy**: if the condition is false on initial render, it will not do anything - the conditional block won't be rendered until the condition becomes true for the first time.

In comparison, `v-show` is much simpler - the element is always rendered regardless of initial condition, with CSS-based toggling.

Generally speaking, `v-if` has higher toggle costs while `v-show` has higher initial render costs. So prefer `v-show` if you need to toggle something very often, and prefer `v-if` if the condition is unlikely to change at runtime.

## `v-if` with `v-for` 

When `v-if` and `v-for` are both used on the same element, `v-if` will be evaluated first. See the [list rendering guide](list#v-for-with-v-if) for details.

::: warning Note
It's **not** recommended to use `v-if` and `v-for` on the same element due to implicit precedence. Refer to [list rendering guide](list#v-for-with-v-if) for details.
:::
