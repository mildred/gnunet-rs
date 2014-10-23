use std::mem::{uninitialized, size_of_val};
use std::fmt::{Show, Formatter};
use std::fmt;
use std::str::from_utf8;
use std::from_str::FromStr;
use std::rand::{Rand, Rng};
use std::hash::Hash;
use std::hash;
use libc::{c_char, c_uint, c_void, size_t};

use ll;

pub struct HashCode {
  data: ll::Struct_GNUNET_HashCode,
}

impl HashCode {
  pub fn hash(buf: &[u8]) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash(buf.as_ptr() as *const c_void, buf.len() as size_t, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }

  pub fn distance(&self, other: &HashCode) -> u32 {
    unsafe {
      ll::GNUNET_CRYPTO_hash_distance_u32(&self.data, &other.data) as u32
    }
  }

  pub fn get_bit(&self, idx: uint) -> bool {
    unsafe {
      ll::GNUNET_CRYPTO_hash_get_bit(&self.data, idx as c_uint) == 1
    }
  }

  pub fn matching_prefix_len(&self, other: &HashCode) -> uint {
    unsafe {
      ll::GNUNET_CRYPTO_hash_matching_bits(&self.data, &other.data) as uint
    }
  }

  pub fn xor_cmp(&self, h1: &HashCode, h2: &HashCode) -> Ordering {
    unsafe {
      match ll::GNUNET_CRYPTO_hash_xorcmp(&h1.data, &h2.data, &self.data) {
        -1  => Less,
        0   => Equal,
        1   => Greater,
        _   => fail!("Invalid value returned by ll::GNUNET_CRYPTO_hash_xorcmp"),
      }
    }
  }
}

impl PartialEq for HashCode {
  fn eq(&self, other: &HashCode) -> bool {
    self.data.bits == other.data.bits
  }
}

impl Eq for HashCode {}

impl Clone for HashCode {
  fn clone(&self) -> HashCode {
    HashCode {
      data: ll::Struct_GNUNET_HashCode {
        bits: self.data.bits,
      },
    }
  }
}

impl Show for HashCode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    unsafe {
      const LEN: uint = 103u;
      assert!(LEN == (size_of_val(&self.data.bits) * 8 + 4) / 5);
      let mut enc: [u8, ..LEN] = uninitialized();
      let res = ll::GNUNET_STRINGS_data_to_string(self.data.bits.as_ptr() as *const c_void,
                                                  size_of_val(&self.data.bits) as size_t,
                                                  enc.as_mut_ptr() as *mut c_char,
                                                  enc.len() as size_t);
      assert!(res.is_not_null());
      from_utf8(enc).unwrap().fmt(f)
    }
  }
}

impl FromStr for HashCode {
  fn from_str(s: &str) -> Option<HashCode> {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      let res = ll::GNUNET_CRYPTO_hash_from_string2(s.as_ptr() as *const i8, s.len() as size_t, &mut ret);
      match res {
        ll::GNUNET_OK => Some(HashCode {
            data: ret,
        }),
        _ => None,
      }
    }
  }
}

impl Rand for HashCode {
  fn rand<R: Rng>(rng: &mut R) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      for u in ret.bits.iter_mut() {
        *u = rng.next_u32();
      };
      HashCode {
        data: ret,
      }
    }
  }
}

impl Add<HashCode, HashCode> for HashCode {
  fn add(&self, rhs: &HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_sum(&self.data, &rhs.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl Sub<HashCode, HashCode> for HashCode {
  fn sub(&self, rhs: &HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_difference(&rhs.data, &self.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl BitXor<HashCode, HashCode> for HashCode {
  fn bitxor(&self, rhs: &HashCode) -> HashCode {
    unsafe {
      let mut ret: ll::Struct_GNUNET_HashCode = uninitialized();
      ll::GNUNET_CRYPTO_hash_xor(&self.data, &rhs.data, &mut ret);
      HashCode {
        data: ret,
      }
    }
  }
}

impl PartialOrd for HashCode {
  fn partial_cmp(&self, other: &HashCode) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for HashCode {
  fn cmp(&self, other: &HashCode) -> Ordering {
    unsafe {
      match ll::GNUNET_CRYPTO_hash_cmp(&self.data, &other.data) {
        -1  => Less,
        0   => Equal,
        1   => Greater,
        _   => fail!("Invalid return from GNUNET_CRYPTO_hash_cmp"),
      }
    }
  }
}

impl<S: hash::Writer> Hash<S> for HashCode {
  fn hash(&self, state: &mut S) {
    self.data.bits.hash(state)
  }
}

/*
impl Iterator<bool> for HashCode {
}
*/

#[test]
fn test_hashcode_to_from_string() {
  let s0: &str = "RMKN0V1JNA3PVC1148D6J10STVG94A8A651N0K849CF1RT6BGF26AMMT14GMDMNRDFSJRJME61KJ31DFBV12R1TPQJE64155132QN5G";
  let hc: Option<HashCode> = FromStr::from_str(s0);
  let s: String = format!("{}", hc.unwrap());
  let s1: &str = s.as_slice();
  println!("s0: {}", s0);
  println!("s1: {}", s1);
  assert!(s0 == s1);
}

#[test]
fn test_hashcode_rand_add_sub() {
  use std::rand::task_rng;

  let mut rng = task_rng();
  let h0: HashCode = rng.gen();
  let h1: HashCode = rng.gen();
  let diff = h1 - h0;
  let sum = h0 + diff;
  assert!(sum == h1);
}
