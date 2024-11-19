## Filters

Though a web-socket subscription model relays can surface events that meet specific criteria on request. 
The means by which these requests maybe submitted are JSON filters objects which can be constructed using a range of attributes, 
including `ids`, `authors`, `kinds` and single letter `tags`, along with timestamps, `since`/`until` and record `limit` for the query.

### Create Filters

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

The following code examples all utilize the `Filters()` along with associated methods to create filter objects and print these in JSON format using the `as_json()` method.

Filtering events based on a specific event ID using `id()`.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-id}}
```

Filtering events by author using `author()`.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-author}}
```

Filtering events based on multiple criteria. In this case, by public key using `pubkey()` and kind using `kind()`.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-kind-pk}}
```

Filtering for specific text strings using `search()`.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-search}}
```

Restricting query results to specific timeframes (using `since()` and `until()`), as well as limiting search results to a maximum of 10 records using `limit()`.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-timeframe}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-limit}}
```

Finally, filtering using hashtags (`hashtag()`), NIP-12 reference tags (`reference()`) and identifiers (`identifiers()`), respectively.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-hashtag}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-reference}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:create-filter-identifier}}
```

</section>

<div slot="title">JavaScript</div>
<section>

The following code examples all utilize the `Filters()` along with associated methods to create filter objects and print these in JSON format using the `asJson()` method.

Filtering events based on a specific event ID using `id()`.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-id}}
```

Filtering events by author using `author()`.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-author}}
```

Filtering events based on multiple criteria. In this case, by public key using `pubkey()` and kind using `kind()`.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-kind-pk}}
```

Filtering for specific text strings using `search()`.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-search}}
```

Restricting query results to specific timeframes (using `since()` and `until()`), as well as limiting search results to a maximum of 10 records using `limit()`.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-timeframe}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-limit}}
```

Finally, filtering using hashtags (`hashtags()`), NIP-12 reference tags (`reference()`) and identifiers (`identifiers()`), respectively.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-hashtag}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-reference}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:create-filter-identifier}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>

### Modify Filters

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Adding more conditions to existing objects can be done by simply calling the relevant method on the instance of the object. 
In this example we create a initial filter with `pubkeys()`, `ids()`, `kinds()` and a single `author()` then modify the object further to include another kind (4) to the existing list of kinds (0, 1).

Similarly, the range of 'remove' methods (e.g. `remove_kinds()`) allow us to take an existing filter and remove unwanted conditions without needed to reconstruct the filter object from scratch.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:modify-filter}}
```

</section>

<div slot="title">JavaScript</div>
<section>

Adding more conditions to existing objects can be done by simply calling the relevant method on the instance of the object. 
In this example we create a initial filter with `pubkeys()`, `ids()`, `kinds()` and a single `author()` then modify the object further to include another kind (4) to the existing list of kinds (0, 1).

Similarly, the range of 'remove' methods (e.g. `removekinds()`) allow us to take an existing filter and remove unwanted conditions without needed to reconstruct the filter object from scratch.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:modify-filter}}
```
</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>

### Other Filter Operations

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

We can parse existing filter JSON object using the `from_json()` method when instantiating a filter object.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:other-parse}}
```

Furthermore, it is possible to create filter records more formally using the `FilterRecord` class.

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:other-record}}
```

To perform a logical test and determine if a given event object matches existing filter conditions the `match_event()` method can be used. 

```python,ignore
{{#include ../../../snippets/python/src/messages/filters.py:other-match}}
```

</section>

<div slot="title">JavaScript</div>
<section>

We can parse existing filter JSON object using the `fromJson()` method when instantiating a filter object.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:other-parse}}
```

To perform a logical test and determine if a given event object matches existing filter conditions the `matchEvent()` method can be used. 

```typescript,ignore
{{#include ../../../snippets/js/src/messages/filters.ts:other-match}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>
