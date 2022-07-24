import test from 'ava'

import { lookup4 } from '../index.js'

test('check wether lookup4 works', async (t) => {
  const data = {
    range: [ 1360405504, 1360405760 ],
    country: 'IT',
    region: '25',
    eu: '1',
    timezone: 'Europe/Rome',
    city: 'Milan',
    ll: [ 45.4722, 9.1922 ],
    metro: 0,
    area: 20
  }
  
  const lookup =await lookup4("81.22.36.183")

  t.deepEqual(data, lookup)

})

test('Check lookup4 function for unexisting ip address', async(t) => {
  const unexisting = await lookup4("0.22.36.183")

  t.is(unexisting, null)
})