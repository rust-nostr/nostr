## Tag

Tags are one of the main element of Nostr event objects and allow for diverse functionality including referencing public keys `p`, relays `r` or even other events `e`. 
The format tags take is an array of strings where the first position in the array is reserved for the tag name and the subsequent strings are the values.

The [Tag](https://docs.rs/nostr/latest/nostr/event/tag/struct.Tag.html) struct and [TagKind](https://docs.rs/nostr/latest/nostr/event/tag/kind/enum.TagKind.html) enum can be used to create and manipulate Tag objects. 

Please refer to the [Standardized Tags](https://github.com/nostr-protocol/nips/tree/master) section of the Nostr Protocol NIP repository for an exhaustive list of tags and their related uses within event kinds.

### Creating Tags

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

There are multiple methods by which we can create tag objects all of which form part of the `Tag` class. The simplest of which are the more commonly used single letter tags. In the example below the `e`, `p`, `a`, `d`, `r` and `t` tags are created passing the relevant object/string values to the tag methods `event()`, `public_key()`, `coordinate()`, `identifier()`, `relay_metadata()` and `hashtag()`, respectively.

```python,ignore
{{#include ../../../snippets/python/src/event/tags.py:single-letter}}
```

For the less commonly used but well defined tags the combination of the `custom()` method is used with an appropriate instance of the `TagKind` class. Please refer to the documentation for a more comprehensive list of the available options.

```python,ignore
{{#include ../../../snippets/python/src/event/tags.py:custom}}
```

Finally, if you are looking to parse lists into tag objects the `parse()` method can be called and passed a list of strings where the first position in the list would represent the tag name and the subsequent strings represent the values. 

```python,ignore
{{#include ../../../snippets/python/src/event/tags.py:parse}}
```

</section>

<div slot="title">JavaScript</div>
<section>

TODO

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

### Serializing and Logical Tests

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Once you have a Tag object, it is relatively straight forward to access the attributes and other related content. The `kind()` method can be used to access the underlying `TagKind` object, the `single_letter_tag()` method returns the `SingleLetterTag` object and `content()` method will return the content of the first value position within the tag (position 1 in the array). 

The `as_standardized()` and `as_vec()` methods will return the tag in both TagStandard (enum) format or as an array of strings, respectively. 

```python,ignore
{{#include ../../../snippets/python/src/event/tags.py:access}}
```

One last point of note is that when processing non-single letter tags it is useful to be able to easily perform tests on these. We can use the `kind()` method to first surface the `TagKind` and then call the relevant "is_x" method (e.g. `is_title()` or `is_summary()` per the example below) to return a boolean result.

```python,ignore
{{#include ../../../snippets/python/src/event/tags.py:logical}}
```

</section>

<div slot="title">JavaScript</div>
<section>

TODO

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
