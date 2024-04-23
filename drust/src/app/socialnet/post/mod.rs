use crate::drust_std::collections::{dstring::DString, dvec::DVec};

pub struct Post {
  pub post_id: usize,
  pub req_id: usize,
  pub user_id: usize,
  pub text: DString,
  pub mentions: DVec<DString>,
  pub media_id: usize,
  pub watermark_id: usize,
  pub timestamp: u128,
  pub post_type: u8,
}