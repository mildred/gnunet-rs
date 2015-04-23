use std::mem::{uninitialized, size_of_val};
use std::fmt;
use std::str::{from_utf8, FromStr};
use std::io::{self, Read, Write};
use libc::{c_void, c_char, size_t};
use byteorder::{self, BigEndian, ReadBytesExt, WriteBytesExt};

use ll;
use Configuration;
use service::{connect, ServiceReader};
use Hello;
use util::io::ReadUtil;

use peerinfo::error::*;

/// The identity of a GNUnet peer.
pub struct PeerIdentity {
  data: ll::Struct_GNUNET_PeerIdentity,
}

impl PeerIdentity {
  pub fn deserialize<R>(r: &mut R) -> Result<PeerIdentity, io::Error> where R: Read {
    let mut ret: PeerIdentity = unsafe { uninitialized() };
    try!(r.read_exact(&mut ret.data.public_key.q_y[..]));
    Ok(ret)
  }

  pub fn serialize<T>(&self, w: &mut T) -> Result<(), io::Error> where T: Write {
    w.write_all(&self.data.public_key.q_y[..])
  }
}

impl FromStr for PeerIdentity {
  type Err = PeerIdentityFromStrError;

  fn from_str(s: &str) -> Result<PeerIdentity, PeerIdentityFromStrError> {
    unsafe {
      let ret: ll::Struct_GNUNET_PeerIdentity = uninitialized();
      let res = ll::GNUNET_STRINGS_string_to_data(s.as_ptr() as *const i8, s.len() as size_t, ret.public_key.q_y.as_ptr() as *mut c_void, ret.public_key.q_y.len() as size_t);
      match res {
        ll::GNUNET_OK => Ok(PeerIdentity {
          data: ret,
        }),
        _ => Err(PeerIdentityFromStrError),
      }
    }
  }
}

/// Iterate over all the currently connected peers.
pub fn iterate_peers(cfg: &Configuration) -> Result<Peers, IteratePeersError> {
  let (sr, mut sw) = try!(connect(cfg, "peerinfo"));
  
  let msg_length = 8u16;
  let mut mw = sw.write_message(msg_length, ll::GNUNET_MESSAGE_TYPE_PEERINFO_GET_ALL);
  mw.write_u32::<BigEndian>(0).unwrap();
  try!(mw.send());
  Ok(Peers {
    service: sr,
  })
} 

/// An iterator over all the currently connected peers.
pub struct Peers {
  service: ServiceReader,
}

impl Iterator for Peers {
  type Item = Result<(PeerIdentity, Option<Hello>), NextPeerError>;

  fn next(&mut self) -> Option<Result<(PeerIdentity, Option<Hello>), NextPeerError>> {
    let (tpe, mut mr) = match self.service.read_message() {
      Err(e)  => return Some(Err(NextPeerError::ReadMessage(e))),
      Ok(x)   => x,
    };
    match tpe {
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO => match mr.read_u32::<BigEndian>() {
        Err(e)  => match e {
          byteorder::Error::UnexpectedEOF => Some(Err(NextPeerError::Disconnected)),
          byteorder::Error::Io(e)         => Some(Err(NextPeerError::Io(e))),
        },
        Ok(x)   => match x == 0 {
          false => Some(Err(NextPeerError::InvalidResponse)),
          true  => match PeerIdentity::deserialize(&mut mr) {
            Err(e)  => Some(Err(NextPeerError::Io(e))),
            Ok(pi)  => {
              Some(Ok((pi, None)))
              /*
               * when we have hello parsing
              match mr.eof() {
                true  => Some(Ok(pi, None)),
                false => {

                },
              }
              */
            },
          },
        },
      },
      ll::GNUNET_MESSAGE_TYPE_PEERINFO_INFO_END => None,
      x => Some(Err(NextPeerError::UnexpectedMessageType(x))),
    }
  }
}

impl fmt::Debug for PeerIdentity {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unsafe {
      const LEN: usize = 52usize;
      assert!(LEN == (size_of_val(&self.data.public_key.q_y) * 8 + 4) / 5);
      let mut enc: [u8; LEN] = uninitialized();
      let res = ll::GNUNET_STRINGS_data_to_string(self.data.public_key.q_y.as_ptr() as *const c_void,
                                                  self.data.public_key.q_y.len() as size_t,
                                                  enc.as_mut_ptr() as *mut c_char,
                                                  enc.len() as size_t);
      assert!(!res.is_null());
      fmt::Display::fmt(from_utf8(&enc).unwrap(), f)
    }
  }
}

impl fmt::Display for PeerIdentity {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(self, f)
  }
}

