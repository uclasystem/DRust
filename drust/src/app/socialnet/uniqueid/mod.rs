use std::sync::Once;

use dashmap::DashMap;
use rand::{distributions::{Distribution, Uniform}, thread_rng};
use super::conf::*;

use crate::{dassert, conf::*};

pub static mut UNIQUE_ID_CACHE: Option<DashMap<usize, bool>> = None;
static INIT_UNIQUEID: Once = Once::new();

pub async fn unique_id() -> usize {
  dassert!(unsafe{SERVER_INDEX} == unsafe{UNIQUE_ID_SERVER_ID}, "Wrong server for unique id");
  INIT_UNIQUEID.call_once(|| {
      unsafe{UNIQUE_ID_CACHE = Some(DashMap::new());}
  });

  let mut rng = thread_rng();
  let range = Uniform::new(0, 1000000000000000000);
  let mut id = range.sample(&mut rng);
  let unique_id_cache = unsafe{UNIQUE_ID_CACHE.as_ref().unwrap()};
  while unique_id_cache.contains_key(&id) {
      id = range.sample(&mut rng);
  }
  unique_id_cache.insert(id, true);
  id
}