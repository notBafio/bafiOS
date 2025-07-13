use alloc::vec::Vec;
use core::borrow::Borrow;
use core::fmt::{self, Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::mem;
pub type DefaultHasher = FnvHasher;

#[derive(Clone)]
pub struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    pub fn new() -> Self {
        FnvHasher {
            state: 0xcbf29ce484222325,
        }
    }
}

impl Default for FnvHasher {
    fn default() -> Self {
        FnvHasher::new()
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        let fnv_prime = 0x100000001b3;
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(fnv_prime);
        }
    }
}

#[derive(Clone)]
pub struct HashMap<K, V, H = DefaultHasher> {
    buckets: Vec<Option<Vec<(K, V)>>>,
    items: usize,
    hasher: H,
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Hash + Eq,
    H: Hasher + Default + Clone,
{
    pub fn new() -> Self {
        const INITIAL_BUCKETS: usize = 16;
        Self::with_capacity(INITIAL_BUCKETS)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        buckets.resize_with(capacity, || None);

        HashMap {
            buckets,
            items: 0,
            hasher: Default::default(),
        }
    }
    fn bucket_for_key<Q: ?Sized>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut hasher = self.hasher.clone();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.buckets.len()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.items >= (self.buckets.len() * 3 / 4) {
            self.resize();
        }

        let bucket_idx = self.bucket_for_key(&key);
        if self.buckets[bucket_idx].is_none() {
            self.buckets[bucket_idx] = Some(Vec::new());
        }

        let bucket = self.buckets[bucket_idx].as_mut().unwrap();
        for (existing_key, existing_value) in bucket.iter_mut() {
            if existing_key == &key {
                return Some(mem::replace(existing_value, value));
            }
        }
        bucket.push((key, value));
        self.items += 1;
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket_idx = self.bucket_for_key(key);
        self.buckets[bucket_idx].as_ref().and_then(|bucket| {
            bucket
                .iter()
                .find(|(k, _)| k.borrow() == key)
                .map(|(_, v)| v)
        })
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket_idx = self.bucket_for_key(key);
        if let Some(bucket) = &mut self.buckets[bucket_idx] {
            if let Some(pos) = bucket.iter().position(|(k, _)| k.borrow() == key) {
                let (_, value) = bucket.remove(pos);
                self.items -= 1;
                return Some(value);
            }
        }
        None
    }
    fn resize(&mut self) {
        let new_size = self.buckets.len() * 2;
        let mut new_buckets = Vec::with_capacity(new_size);
        new_buckets.resize_with(new_size, || None);
        for bucket in self.buckets.drain(..) {
            if let Some(vec) = bucket {
                for (key, value) in vec {
                    let mut hasher = self.hasher.clone();
                    key.hash(&mut hasher);
                    let idx = (hasher.finish() as usize) % new_size;

                    if new_buckets[idx].is_none() {
                        new_buckets[idx] = Some(Vec::new());
                    }
                    new_buckets[idx].as_mut().unwrap().push((key, value));
                }
            }
        }

        self.buckets = new_buckets;
    }
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            bucket_iter: self.buckets.iter(),
            vec_iter: None,
        }
    }
}
pub struct Iter<'a, K, V> {
    bucket_iter: core::slice::Iter<'a, Option<Vec<(K, V)>>>,
    vec_iter: Option<core::slice::Iter<'a, (K, V)>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut vec_iter) = self.vec_iter {
                if let Some((k, v)) = vec_iter.next() {
                    return Some((k, v));
                }
            }
            match self.bucket_iter.next() {
                Some(Some(bucket)) => {
                    self.vec_iter = Some(bucket.iter());
                    continue;
                }
                Some(None) => continue,
                None => return None,
            }
        }
    }
}
impl<'a, K, V, H> IntoIterator for &'a HashMap<K, V, H>
where
    K: Hash + Eq,
    H: Hasher + Default + Clone,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<K, V, H> Debug for HashMap<K, V, H>
where
    K: Hash + Eq + Debug,
    V: Debug,
    H: Hasher + Default + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (k, v) in self.iter() {
            map.entry(k, v);
        }
        map.finish()
    }
}
