use crate::IpBlockRecord;

pub fn item_binary_search(list: &Vec<IpBlockRecord>, item: u32) -> isize {
  let mut low: usize = 0;
  let mut high: usize = list.len() - 1;

  loop {
    let index: usize = (((((high as f32) - (low as f32)) / 2.0) as f32).round() as usize) + low;
    if item < list[index].0 {
      if index == high && index == low {
        return -1;
      } else if index == high {
        high = low;
      } else {
        high = index;
      }
    } else if item >= list[index].0 && (index == (list.len() - 1) || item < list[index + 1].0) {
      return index as isize;
    } else {
      low = index;
    }
  }
}

pub fn ip_string_to_number(ip: String) -> u32 {
  ip.split('.')
    .map(|x| x.parse::<u32>().unwrap())
    .enumerate()
    .fold(0, |val, (index, acc)| {
      val + acc * 256_u32.pow(3 - index as u32)
    })
}

pub fn file_binary_search(list: &Vec<u32>, item: u32) -> isize {
  let mut low: usize = 0;
  let mut high: usize = list.len() - 1;

  loop {
    let index: usize = (((((high as f32) - (low as f32)) / 2.0) as f32).round() as usize) + low;

    if item < list[index] {
      if index == high && index == low {
        return -1;
      } else if index == high {
        high = low;
      } else {
        high = index;
      }
    } else if item >= list[index] && (index == (list.len() - 1) || item < list[index + 1]) {
      return index as isize;
    } else {
      low = index;
    }
  }
}

pub fn get_next_ip_from_u32(list: &Vec<u32>, index: isize, current_next_ip: u32) -> u32 {
  if index < (list.len() - 1) as isize {
    list[(index as usize) + 1]
  } else {
    current_next_ip
  }
}

pub fn get_next_ip_from_list(list: &Vec<IpBlockRecord>, index: isize, current_next_ip: u32) -> u32 {
  if index < (list.len() - 1) as isize {
    list[(index as usize) + 1].0
  } else {
    current_next_ip
  }
}
