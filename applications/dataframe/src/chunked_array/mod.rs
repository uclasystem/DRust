pub mod conf;
pub mod chunk;
pub mod utils;

use std::{borrow::Borrow, collections::HashMap, marker::PhantomData, sync::Arc};

use dashmap::DashMap;
use fnv::FnvHashMap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator, IntoParallelRefMutIterator, IndexedParallelIterator};
use serde::{ Deserialize, Serialize };
use utils::*;

use super::{prelude::*, utils::*};

pub struct ChunkedArray<T> {
    pub(crate) field: Field,
    pub(crate) chunks: Vec<Chunk>,
    phantom: PhantomData<T>,
}

impl<T> ChunkedArray<T> {
    pub fn chunks_num(&self) -> usize {
        unsafe{self.chunks.len()}
    }

    /// Combined length of all the chunks.
    pub fn len(&self) -> usize {
        let mut len = 0;
        for i in 0..self.chunks_num() {
            len += self.chunks[i].len;
        }
        len
    }

    /// Get data type of ChunkedArray.
    pub fn dtype(&self) -> &DataType {
        self.field.data_type()
    }

    /// Get the index of the chunk and the index of the value in that chunk
    #[inline]
    pub(crate) fn index_to_chunked_index_simple(
        &self,
        index: usize,
        chunk_capcity: usize
    ) -> (usize, usize) {
        (index / chunk_capcity, index % chunk_capcity)
    }

    /// Name of the ChunkedArray.
    pub fn name(&self) -> &str {
        self.field.name()
    }

    /// Rename this ChunkedArray.
    pub fn rename(&mut self, name: &str) {
        self.field.rename(name);
    }

    pub fn get_ref<'a>(&'a self) -> &'a Vec<Chunk> {
        self.chunks.as_ref()
    }

    pub fn get_mut_ref<'a>(&'a mut self) -> &'a mut Vec<Chunk> {
        self.chunks.as_mut()
    }

    pub fn into_raw(self) -> (Field, Vec<Chunk>) {
        (self.field, self.chunks)
    }

    pub fn from_raw(field: Field, chunks: Vec<Chunk>) -> Self {
        Self {
            field,
            chunks,
            phantom: PhantomData,
        }
    }
}

impl<T> ChunkedArray<T> where T: PolarsDataType {}

impl<T> ChunkedArray<T> where T: PrimitiveType + Sync + Send {
    pub fn push_value(&mut self, item: Vec<u8>, row_id: usize) {
        if item.len() != T::get_bit_width() / 8 {
            panic!("Item size does not match array size");
        }
        let (chunk_id, chunk_index) = self.index_to_chunked_index_simple(
            row_id,
            CHUNK_SIZE / (T::get_bit_width() / 8)
        );
        let chunks_mut: &mut Vec<Chunk> = self.chunks.as_mut();
        let chunk = chunks_mut.get_mut(chunk_id).unwrap();
        let return_index = chunk.push(&item);
        if return_index != chunk_index {
            panic!("Chunk index does not match");
        }
    }

    /// Push Item to Array
    pub fn push(&mut self, item: Option<Vec<u8>>, row_id: usize) {
        match item {
            None => panic!("Item is None"),
            Some(item) => self.push_value(item, row_id),
        }
    }

    // TODO: consider null values
    pub fn take_unchecked(&mut self,
        src_chunks: &Vec<Chunk>,
        indices: &Vec<usize>,
    ) {
        let local_src_chunks = src_chunks;

        let chunk_capacity = CHUNK_SIZE / (T::get_bit_width() / 8);
        let capacity = indices.len();
        let chunks_mut: &mut Vec<Chunk> = self.chunks.as_mut();
        chunks_mut
        .par_iter_mut().enumerate().for_each(|(new_chunk_idx, new_chunk)| {
            if new_chunk_idx < capacity / chunk_capacity {
                new_chunk.len = chunk_capacity;
            } else {
                new_chunk.len = capacity % chunk_capacity;
            }

            let st_id = new_chunk_idx * chunk_capacity;
            let ed_id = st_id + new_chunk.len;

            for new_id in st_id..ed_id {
                let id = indices[new_id];
                let (chunk_idx, idx) = (id/chunk_capacity, id%chunk_capacity);
                let new_idx = new_id - st_id;
                let arr = local_src_chunks.get(chunk_idx).unwrap();

                let new_offset = new_idx * new_chunk.element_size;
                new_chunk.buffer[new_offset..new_offset + new_chunk.element_size].copy_from_slice(
                    arr.buffer[
                        idx * arr.element_size..idx * arr.element_size + arr.element_size
                    ].as_ref()
                );
            }
        });

        println!("Finish take");
    }

    /// Get a single value. Beware this is slow.
    pub fn rawget(&self, index: usize) -> AnyType {
        let (chunk_idx, idx) = self.index_to_chunked_index_simple(
            index,
            CHUNK_SIZE / (T::get_bit_width() / 8)
        );
        let chunks_ref: &Vec<Chunk> = self.chunks.as_ref();
        let arr = &chunks_ref[chunk_idx];
        // TODO: insert types
        Chunk::get_any(arr, idx, T::get_data_type())
    }

    pub async fn get(&self, index: usize) -> AnyType {
        let (chunk_idx, idx) = self.index_to_chunked_index_simple(
            index,
            CHUNK_SIZE / (T::get_bit_width() / 8)
        );
        let array: &Vec<Chunk> = self.chunks.as_ref();
        let c = array.get(chunk_idx).unwrap();
        let v = Chunk::get_any(c, idx, T::get_data_type());
        v
    }

    pub fn get_four_bytes(&self, index: usize) -> [u8; 4] {
        let (chunk_idx, idx) = self.index_to_chunked_index_simple(
            index,
            CHUNK_SIZE / (T::get_bit_width() / 8)
        );
        let chunks_ref: &Vec<Chunk> = self.chunks.as_ref();
        let arr = &chunks_ref[chunk_idx];
        arr.get_four_bytes(idx)
    }

    /// Build Array from name:
    pub fn new_from_name(name: &str, line_cnt: usize) -> Self {
        let field = Field::new(name, T::get_data_type(), true);
        let element_size = std::cmp::max(1, T::get_bit_width()/8);
        let chunk_num = line_cnt * element_size / CHUNK_SIZE + 1;
        let mut vec = Vec::with_capacity(chunk_num);
        for _ in 0..chunk_num {
            vec.push(Chunk::new(element_size));
        }
        Self {
            field,
            chunks: vec,
            phantom: PhantomData,
        }
    }
}