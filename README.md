# Fuzzr

Fuzzr is an incredibly fast fuzzy searcher compiled for browser use. This
library is written in rust and utilizes the popular skim algorithm used in the
skim command line search. The algorithm is provided by the
[fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher) crate by
[lotabout](https://github.com/lotabout). I merely provided the wasm bindings.

## Usage

``` js
import Fuzzr from 'fuzzr'

const collection = ['Apple', 'Orange', 'Pineapple', 'Pear']
const fuzzr = new Fuzzr(collection, {
  wrapResultsWith: ['<strong>', '</strong>']
})

fuzzr.search('Ale')
/*
[
  {
    item: 'Apple',
    score: 42,
    formatted: '<strong>A</strong>pp<strong>le</strong>'
  },
  {
    item: 'Pineapple',
    score: 42,
    formatted: 'Pine<strong>a</strong>pp<strong>le</strong>'
  }
]
*/
```

Results are ordered in descending order by score. If the score is equal, the
order of the elements in the collection is preserved.
