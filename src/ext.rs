use std::{convert::TryInto, ops::Deref};

use mesagisto_client::{OkExt, OptionExt, db::DB as INNER};


#[derive(Singleton, Default)]
pub struct Db{}
impl Db {
  #[inline]
  pub fn put_msg_id_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.to_be_bytes().to_vec(),
      true,
    )
  }
  // no reverse
  #[inline]
  pub fn put_msg_id_ir_0(&self, target: &i64, uid: &i32, id: &i32) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.to_be_bytes().to_vec(),
      false,
    )
  }
  #[inline]
  pub fn put_msg_id_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.clone(),
      id.to_be_bytes().to_vec(),
      true,
    )
  }
  #[inline]
  pub fn put_msg_id_ir_1(&self, target: &i64, uid: &Vec<u8>, id: &i32) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.clone(),
      id.to_be_bytes().to_vec(),
      false,
    )
  }
  #[inline]
  pub fn put_msg_id_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      true,
    )
  }
  #[inline]
  pub fn put_msg_id_ir_2(&self, target: &i64, uid: &i32, id: &Vec<u8>) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      false,
    )
  }
  #[inline]
  pub fn put_msg_id_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      true,
    )
  }
  #[inline]
  pub fn put_msg_id_ir_3(&self, target: &u64, uid: &u64, id: &Vec<u8>) -> anyhow::Result<()> {
    INNER.put_msg_id(
      target.to_be_bytes().to_vec(),
      uid.to_be_bytes().to_vec(),
      id.clone(),
      false,
    )
  }
  #[inline]
  pub fn get_msg_id_1(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<i32>> {
    let be_bytes = match INNER.get_msg_id(&target.to_be_bytes().to_vec(), id)? {
      Some(v) => match v.len() {
        4 => v,
        _ => return Ok(None),
      },
      None => return Ok(None),
    };
    i32::from_be_bytes(be_bytes.try_into().unwrap()).some().ok()
  }
  #[inline]
  pub fn get_msg_id_2(&self, target: &i64, id: &Vec<u8>) -> anyhow::Result<Option<Vec<u8>>> {
    INNER.get_msg_id(&target.to_be_bytes().to_vec(), id)
  }
}
impl Deref for Db {
  type Target = mesagisto_client::db::Db;

  fn deref(&self) -> &Self::Target {
    &*INNER
  }
}
