//!
//!
//!
//!
//!
//!

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use venus::hash::SHA1;
use venus::internal::object::types::ObjectType;
use dashmap::DashMap;


#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CacheObject {
    pub base_offset: usize,
    pub base_ref: SHA1,
    pub data_decompress: Vec<u8>,
    pub object_type: ObjectType,
    pub offset: usize,
    pub hash: SHA1,
}

pub trait _Cache {
    fn new(size: Option<usize>, tmp_path: Option<PathBuf>) -> Self
    where
        Self: Sized;
    fn get_hash(&self, offset: usize) -> Option<SHA1>;
    fn insert(&self, offset: usize, hash: SHA1, obj: CacheObject);
    fn get_by_offset(&self, offset: usize) -> Option<Arc<CacheObject>>;
    fn get_by_hash(&self, h: SHA1) -> Option<Arc<CacheObject>>;
}

#[allow(unused)]
pub struct Caches {
    map_offset: DashMap<usize, SHA1>,
    map_hash: HashMap<SHA1, Arc<CacheObject>>,
    mem_size: usize,
    tmp_path: PathBuf,
}

impl CacheObject {}

impl Caches {
    
}

impl _Cache for Caches {
    fn new(size: Option<usize>, tmp_path: Option<PathBuf>) -> Self
    where
        Self: Sized,
    {
        Caches {
            map_offset: DashMap::new(),
            map_hash: HashMap::new(),
            mem_size: size.unwrap_or(0),
            tmp_path: tmp_path.unwrap_or_default(),
        }
    }
    fn get_hash(&self, offset: usize) -> Option<SHA1> {
        self.map_offset.get(&offset).map(|x| *x)
    }
    fn insert(&self, offset: usize, hash: SHA1, obj: CacheObject) {
        self.map_offset.insert(offset, hash);
        unimplemented!()
    }
    fn get_by_offset(&self, offset: usize) -> Option<Arc<CacheObject>> {
        match self.map_offset.get(&offset) {
            Some(x) => self.get_by_hash(*x),
            None => None,
        }
    }
    fn get_by_hash(&self, hash: SHA1) -> Option<Arc<CacheObject>> {
        unimplemented!()
    }
}


#[cfg(test)]
mod test{

}