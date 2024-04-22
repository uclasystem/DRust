use dashmap::{mapref::entry::Entry, DashMap};
use serde::{ Serialize, Deserialize };
use tarpc::context;
use enum_dispatch::enum_dispatch;
use num::{ Num, NumCast };
use rayon::prelude::*;
use std::{ borrow::Borrow, collections::HashMap, hash::Hash, sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::{ Duration, SystemTime } };
use crate::{conf::*, dprintln, drust_std::{alloc::LOCAL_ALLOCATOR, collections::dvec::{DVec, DVecRef}, thread::{dscope_spawn, dspawn, dspawn_to}}};

use self::utils::to_chunked_index;

use super::super::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum AggType {
    Sum,
    Min,
    Max,
    Mean,
}

async fn agg_min(datatype: DataType, mut src_series: DVecRef<'_, Chunk>, mut indices: DVecRef<'_, usize>, mut groups: DVecRef<'_, usize>) -> DVec<Chunk> {
    let src_ref = src_series.as_ref();
    let indices_ref = indices.as_ref();
    let groups_ref = groups.as_ref();

    let mut new_series = Series::new_from_name(DataType::Float64, "agg", indices_ref.len());
    let (mut field, mut array) = new_series.into_raw();
    let local_src_array = src_ref;
    
    
    let chunk_num = array.len();
    let chunk_capacity = CHUNK_SIZE / (Float64Type::get_bit_width() / 8);
    let src_chunk_capacity = CHUNK_SIZE / datatype_size(&datatype);
    let array_mut = array.as_mut();
    array_mut
        .par_iter_mut()
        .zip(0..chunk_num)
        .for_each(|(chunk, idx)| {
            let st = idx * chunk_capacity;
            let ed = if idx < chunk_num - 1 { st + chunk_capacity } else { indices_ref.len() };
            let chunk_vec: Vec<f64> = indices_ref[st..ed]
                .par_iter()
                .zip(st..ed)
                .map(|(start_group_idx, indices_idx)| {
                    let end_group_idx = if indices_idx < indices_ref.len() - 1 { indices_ref[indices_idx + 1] } else { groups_ref.len() };
                    let mut min: Option<f64> = None;
                    for i in groups_ref[*start_group_idx..end_group_idx].iter() {
                        let (src_chunk_idx, src_idx) = to_chunked_index(*i, src_chunk_capacity);
                        let chunk_ref = local_src_array.get(src_chunk_idx).unwrap();
                        let v = chunk_ref.raw_get(src_idx);
                        let value = match datatype {
                            DataType::Int32 => {convert_bytes_to_u32(v) as f64},
                            DataType::Float64 => {f64::from_bits(convert_bytes_to_u64(v))},
                            _ => {unimplemented!()}
                        };
                        min = match min {
                            Some(min) => {
                                if min < value { Some(min) } else { Some(value) }
                            }
                            None => Some(value),
                        };
                    }
                    min.unwrap()
                })
                .collect();
            unsafe {
                chunk.len = ed - st;
                let src_ptr = chunk_vec.as_ptr() as *const u8;
                let dst_ptr = chunk.buffer.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, chunk.len * chunk.element_size);
            }
        });
   

    drop(indices_ref);
    drop(groups_ref);
    drop(groups);
    drop(indices);
    
    array

}

async fn agg_sum(datatype: DataType, mut src_series: DVecRef<'_, Chunk>, mut indices: DVecRef<'_, usize>, mut groups: DVecRef<'_, usize>) -> DVec<Chunk> {
    let src_ref = src_series.as_ref();
    let indices_ref = indices.as_ref();
    let groups_ref = groups.as_ref();

    let mut new_series = Series::new_from_name(DataType::Float64, "agg", indices_ref.len());
    let (mut field, mut array) = new_series.into_raw();
    let local_src_array = src_ref;
    
    
    let chunk_num = array.len();
    println!("chunk num: {}", chunk_num);
    let chunk_capacity = CHUNK_SIZE / (Float64Type::get_bit_width() / 8);
    let src_chunk_capacity = CHUNK_SIZE / datatype_size(&datatype);
    let array_mut = array.as_mut();
    array_mut
        .par_iter_mut()
        .zip(0..chunk_num)
        .for_each(|(chunk, idx)| {
            let st = idx * chunk_capacity;
            let ed = if idx < chunk_num - 1 { st + chunk_capacity } else { indices_ref.len() };
            let chunk_vec: Vec<f64> = indices_ref[st..ed]
                .par_iter()
                .zip(st..ed)
                .map(|(start_group_idx, indices_idx)| {
                    let end_group_idx = if indices_idx < indices_ref.len() - 1 { indices_ref[indices_idx + 1] } else { groups_ref.len() };
                    let mut sum: f64 = 0.;
                    for i in groups_ref[*start_group_idx..end_group_idx].iter() {
                        let (src_chunk_idx, src_idx) = to_chunked_index(*i, src_chunk_capacity);
                        let chunk_ref = local_src_array.get(src_chunk_idx).unwrap();
                        let v = chunk_ref.raw_get(src_idx);
                        match datatype {
                            DataType::Int32 => {
                                let value = convert_bytes_to_u32(v) as f64;
                                sum = sum + value;
                            },
                            DataType::Float64 => {
                                let value = f64::from_bits(convert_bytes_to_u64(v));
                                sum = sum + value;
                            },
                            _ => unimplemented!(),
                        }
                    }
                    sum
                })
                .collect();
            unsafe {
                chunk.len = ed - st;
                let src_ptr = chunk_vec.as_ptr() as *const u8;
                let dst_ptr = chunk.buffer.as_mut_ptr();
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, chunk.len * chunk.element_size);
            }
            // println!("chunk len: {}", chunk.len);
        });
   

    drop(indices_ref);
    drop(groups_ref);
    drop(groups);
    drop(indices);
    
    array
}

async fn agg_mean(datatype: DataType, src_series: DVecRef<'_, Chunk>, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> DVec<Chunk> {
    unimplemented!()
}

async fn agg_max(datatype: DataType, src_series: DVecRef<'_, Chunk>, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> DVec<Chunk> {
    unimplemented!()
}


#[enum_dispatch(Series)]
pub trait NumericAggSync {
    async fn agg_min(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        unimplemented!()
    }

    async fn agg_sum(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        unimplemented!()
    }
    
    async fn agg_mean(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        unimplemented!()
    }

    async fn agg_max(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        unimplemented!()
    }
}


impl NumericAggSync for BooleanChunked {}

impl<T> NumericAggSync
    for ChunkedArray<T>
    where T: PolarsNumericType + Sync, T::Native: std::ops::Add<Output = T::Native> + Num + NumCast
{
    async fn agg_min(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        let src_ref = self.get_ref();
        let datatype = self.dtype().clone();
        let array = dspawn_to(agg_min(datatype, src_ref, indices, groups), GLOBAL_HEAP_START + WORKER_UNIT_SIZE * self.chunks.server_idx()).await.unwrap();
        let field = Field::new(self.name(), DataType::Float64, true);
        Series::from_raw(field, array)
    }

    async fn agg_sum(&self, indices: DVecRef<'_, usize>, groups: DVecRef<'_, usize>) -> Series {
        let src_ref = self.get_ref();
        let datatype = self.dtype().clone();
        let array = dspawn_to(agg_sum(datatype, src_ref, indices, groups), GLOBAL_HEAP_START + WORKER_UNIT_SIZE * self.chunks.server_idx()).await.unwrap();
        let field = Field::new(self.name(), DataType::Float64, true);
        Series::from_raw(field, array)
    }


}




async fn groupby_work(mut datatypes: DVec<DataType>, mut refs: DVec<DVecRef<'_, Chunk>>) -> (DVec<usize>, DVec<usize>, DVec<usize>) {

    let datatypes_ref = datatypes.as_ref();
    for dtype in datatypes_ref {
        if datatype_size(dtype) != 4 {
            panic!("not implemented!");
        }
    }

    dprintln!("groupby work len: {}", datatypes.len());

    let mut total_len = 0;
    for i in 0..refs[0].len() {
        total_len += refs[0][i].len();

    }
    let capacity = total_len * refs.len() / 6;

    let mut hash_tbl: DashMap<Vec<[u8; 4]>, Vec<usize>> = DashMap::with_capacity(capacity);
    let hashtbl_arc = Arc::new(hash_tbl); 

    let chunks_num = refs[0].len();
    let mut threads_num = if chunks_num < 16 { chunks_num } else { 16 };
    // let threads_num = 1;
    let mut current_chunk_id = 0;

    dprintln!("total len: {}", total_len);

    std::thread::scope(|s| {

    let mut handles = Vec::with_capacity(threads_num);

    for thread_id in 0..threads_num {

        let keys_num = datatypes.len();
        let unit_chunk_num = (chunks_num - current_chunk_id) / (threads_num - thread_id);    
        let st = current_chunk_id;
        let ed = current_chunk_id + unit_chunk_num;
        current_chunk_id = ed;
        let hash_tbl_ref = Arc::clone(&hashtbl_arc);
        
        let mut svec = Vec::new();
        let siz = refs.len();
        for i in 0..siz {
            // dprintln!("thread id: {}, i: {}, clone reference", thread_id, i);
            let mut r = refs[i].clone();
            svec.push(r);
        }

        handles.push(s.spawn( move || {
            for chunk_idx in st..ed {
                let mut chunk_vec = Vec::with_capacity(svec.len());
                for vecref in &mut svec {
                    // dprintln!("thread id: {}, chunk idx: {}, start", thread_id, chunk_idx);
                    let chunk = &vecref[chunk_idx];
                    // dprintln!("thread id: {}, chunk idx: {}, chunk buffer", thread_id, chunk_idx);
                    let mut chunk_buffer = &chunk.buffer;
                    chunk_vec.push((chunk, chunk_buffer));
                }
                // dprintln!("thread id: {}, chunk idx: {}, start to group", thread_id, chunk_idx);
                let total_capacity = CHUNK_SIZE / 4;
                let start_index = chunk_idx * total_capacity;
                let len = chunk_vec[0].0.len();
                for inner_idx in 0..len {
                    let mut v = Vec::with_capacity(keys_num);
                    for (chunk, chunk_buffer) in &chunk_vec {
                        v.push(chunk.get_four_bytes_from_buffer(inner_idx, chunk_buffer));
                    }
                    let idx_a = inner_idx + start_index;
                    let entry_vec = hash_tbl_ref.entry(v);
                    match entry_vec {
                        Entry::Occupied(mut entry) => {
                            let indexes = entry.get_mut();
                            indexes.push(idx_a);
                        },
                        Entry::Vacant(entry) => {
                            entry.insert(vec![idx_a]);
                        }
                    }
                }
            }
        }
        ))
    }
    for handle in handles {
        handle.join().unwrap();
    }
    });
    hash_tbl = Arc::try_unwrap(hashtbl_arc).unwrap();
    
    let len = hash_tbl.len();
    let mut keyv = DVec::with_capacity(len);
    let mut indv = DVec::with_capacity(len);
    let mut groupv = DVec::with_capacity(total_len);
    unsafe{
        keyv.set_len(len);
        indv.set_len(len);
        groupv.set_len(total_len);
    }
    let keyv_ptr = keyv.as_ptr() as *const u8 as usize;
    let indv_ptr = indv.as_ptr() as *const u8 as usize;
    let groupv_ptr = groupv.as_ptr() as *const u8 as usize;

    let current_index = AtomicUsize::new(0);
    
    println!("hash_tbl len: {}", hash_tbl.len());
    hash_tbl.into_par_iter().for_each(|entry| {
        let indexes = entry.1;
        let mut old_index = current_index.load(Ordering::SeqCst);
        let mut groupv_idx = (old_index & ((1<<32)-1)) + indexes.len();
        let mut keyv_idx = (old_index >> 32) + 1;
        let mut new_index = keyv_idx << 32 | groupv_idx;
        while current_index.compare_exchange(old_index, new_index, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            old_index = current_index.load(Ordering::SeqCst);
            groupv_idx = (old_index & ((1<<32)-1)) + indexes.len();
            keyv_idx = (old_index >> 32) + 1;
            new_index = keyv_idx << 32 | groupv_idx;
        }
        let keyv_idx_ptr = keyv_ptr + (keyv_idx - 1) * std::mem::size_of::<usize>();
        let indv_idx_ptr = indv_ptr + (keyv_idx - 1) * std::mem::size_of::<usize>();
        let groupv_idx_ptr = groupv_ptr + (groupv_idx - indexes.len()) * std::mem::size_of::<usize>();
        unsafe {
            *(keyv_idx_ptr as *mut usize) = *(indexes.get_unchecked(0));
            *(indv_idx_ptr as *mut usize) = groupv_idx - indexes.len();
            
            std::ptr::copy_nonoverlapping(indexes.as_ptr(), groupv_idx_ptr as *mut usize, indexes.len());
        }
    });
    println!("current index: {}", current_index.load(Ordering::SeqCst));

    println!("first 4 bytes of keyv: {:?}", keyv[0]);
    dprintln!("addr of keyv: {:p}", keyv.as_ptr());
    dprintln!("addr of indv: {:p}", indv.as_ptr());

    (keyv, indv, groupv)

}


#[enum_dispatch(Series)]
trait IntoGroupTuples {
    async fn group_tuples(
        &self,
        datatypes: DVec<DataType>,
        arrays: DVec<DVecRef<'_, Chunk>>
    ) -> (DVec<usize>, DVec<usize>, DVec<usize>) {
        unimplemented!()
    }
}

impl<T> IntoGroupTuples for ChunkedArray<T> where T: PolarsIntegerType, T::Native: Eq + Hash {
    async fn group_tuples(
        &self,
        datatypes: DVec<DataType>,
        arrays: DVec<DVecRef<'_, Chunk>>
    ) -> (DVec<usize>, DVec<usize>, DVec<usize>) {
        let re = dscope_spawn(groupby_work(datatypes, arrays)).await;
        re
    }
}

impl IntoGroupTuples for Float64Chunked {}
impl IntoGroupTuples for Float32Chunked {}
impl IntoGroupTuples for BooleanChunked {}

impl DataFrame {
    /// Group DataFrame using a Series column.
    ///
    /// # Example
    ///
    /// fn groupby_sum(df: &DataFrame) -> Result<DataFrame> {
    ///     df.groupby("column_name")?
    ///     .select("agg_column_name")
    ///     .sum()
    /// }
    pub async fn groupby(&self, by: Vec<String>) -> Result<GroupBy, PolarsError> {
        let main_name = by.get(0).unwrap();
        dprintln!("groupby: {}", main_name);
        let siz = by.len();
        let mut datatypes = DVec::with_capacity(siz);
        let mut refs = DVec::with_capacity(siz);
        for name in &by {
            if let Some(s) = self.column(name) {
                dprintln!("groupby: {}", name);
                datatypes.push(s.dtype().clone());
                refs.push(s.get_ref());
            } else {
                return Err(PolarsError::NotFound);
            };
        }

        let s = self.column(main_name).unwrap();

        let groups = s.group_tuples(datatypes, refs).await;
        Ok(GroupBy {
            df: self,
            by,
            groups,
            selection: None,
        })
    }
}

// #[derive(Clone)]
pub struct GroupBy<'a> {
    df: &'a DataFrame,
    /// By which column should the grouping operation be performed.
    pub by: Vec<String>,
    // [first idx, [other idx]]
    // groups: Vec<(usize, Vec<usize>)>,
    groups: (DVec<usize>, DVec<usize>, DVec<usize>),
    selection: Option<String>,
}

impl<'a> GroupBy<'a> {
    /// Select the column by which the determine the groups.
    pub fn select(&mut self, name: &str) {
        self.selection = Some(name.to_string());
    }

    pub fn groups(&self) -> &(DVec<usize>, DVec<usize>, DVec<usize>) {
        &self.groups
    }

    pub async fn keys(&self) -> Vec<Series> {
        let mut ks = vec![];

        let mut drop_names = vec![];
        for col_name in self.df.columns() {
            if !self.by.contains(&col_name.to_string()) {
                drop_names.push(col_name.to_string());
            }
        }

        let key_id_dvec = &self.groups.0;

        let f = unsafe {
            self.df.take_iter_unchecked(
                key_id_dvec,
                key_id_dvec.len(),
                drop_names
            ).await
        };

        for k in f.columns {
            ks.push(k);
        }
        ks
    }

    pub async fn sum_series(&self, name: &str) -> Result<Series, PolarsError> {
        let agg_col = self.df.column(name).ok_or(PolarsError::NotFound)?;
        let new_name = format!("{}_sum", name);
        
        let indices = self.groups.1.as_dref();
        let groups = self.groups.2.as_dref();

        let mut agg = agg_col.agg_sum(indices, groups).await;
        agg.rename(&new_name);
        Ok(agg)
    }
    
    pub async fn min_series(&self, name: &str) -> Result<Series, PolarsError> {
        let agg_col = self.df.column(name).ok_or(PolarsError::NotFound)?;
        let new_name = format!("{}_sum", name);
        
        let indices = self.groups.1.as_dref();
        let groups = self.groups.2.as_dref();

        let mut agg = agg_col.agg_min(indices, groups).await;
        agg.rename(&new_name);
        Ok(agg)
    }

}
