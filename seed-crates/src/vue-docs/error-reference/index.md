
import  from 'vue'
import  from './errors.data.ts'
import ErrorsTable from './ErrorsTable.vue'

const highlight = ref()
onMounted(() => )

# Production Error Code Reference 

## Runtime Errors 

In production builds, the 3rd argument passed to the following error handler APIs will be a short code instead of the full information string:

- [`app.config.errorHandler`](/api/application#app-config-errorhandler)
- [`onErrorCaptured`](/api/composition-api-lifecycle#onerrorcaptured) (Composition API)
- [`errorCaptured`](/api/options-lifecycle#errorcaptured) (Options API)

The following table maps the codes to their original full information strings.

## Compiler Errors 

The following table provides a mapping of the production compiler error codes to their original messages.

