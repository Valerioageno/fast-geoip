/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface IpInfo {
  range: Array<number>
  country: string
  region: string
  eu: string
  timezone: string
  city: string
  ll: Array<number>
  metro: number
  area: number
}
export function lookup4(ipv4: string): Promise<IpInfo | null>
