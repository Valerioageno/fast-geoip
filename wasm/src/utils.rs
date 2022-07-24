pub fn ip_string_to_number(ip: String) -> u32 {
  ip.split('.')
    .map(|x| x.parse::<u32>().unwrap())
    .enumerate()
    .fold(0, |val, (index, acc)| {
      val + acc * 256_u32.pow(3 - index as u32)
    })
}
