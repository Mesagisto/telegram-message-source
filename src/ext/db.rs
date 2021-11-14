use std::convert::TryInto;

use mesagisto_client::{db::Db, OkExt, OptionExt};

pub trait DbExt {
  fn put_msg_id_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()>;
  fn put_msg_id_ir_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()>;

  fn put_msg_id_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()>;
  fn put_msg_id_ir_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()>;

  fn put_msg_id_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()>;
  fn put_msg_id_ir_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()>;

  fn put_msg_id_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()>;
  fn put_msg_id_ir_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()>;

  fn get_msg_id_1(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<i32>>;
  fn get_msg_id_2(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<Vec<u8>>>;
}

impl DbExt for Db {
  #[inline]
  fn put_msg_id_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.to_be_bytes().to_vec(),
      true,
    )
  }
  // no reverse
  #[inline]
  fn put_msg_id_ir_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.to_be_bytes().to_vec(),
      false,
    )
  }
  #[inline]
  fn put_msg_id_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.clone(),
      id.to_be_bytes().to_vec(),
      true,
    )
  }
  #[inline]
  fn put_msg_id_ir_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.clone(),
      id.to_be_bytes().to_vec(),
      false,
    )
  }
  #[inline]
  fn put_msg_id_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      true,
    )
  }
  #[inline]
  fn put_msg_id_ir_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      false,
    )
  }
  #[inline]
  fn put_msg_id_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      true,
    )
  }
  #[inline]
  fn put_msg_id_ir_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()> {
    self.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      false,
    )
  }
  #[inline]
  fn get_msg_id_1(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<i32>> {
    let be_bytes = match self.get_msg_id(&target.to_be_bytes().to_vec(), id)? {
      Some(v) => match v.len() {
        4 => v,
        _ => return Ok(None),
      },
      None => return Ok(None),
    };
    i32::from_be_bytes(be_bytes.try_into().unwrap()).some().ok()
  }
  #[inline]
  fn get_msg_id_2(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<Vec<u8>>> {
    self.get_msg_id(&target.to_be_bytes().to_vec(), id)
  }
}
