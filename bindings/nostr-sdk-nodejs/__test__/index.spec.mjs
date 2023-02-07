import test from 'ava'

import { Keys } from '../index.js'

test('should be able to generate', (t) => {
  t.true(Keys.generate() instanceof Keys)
  t.is(typeof Keys.generate().publicKey(), 'string')
})
