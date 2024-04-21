use super::{chunk::Chunk, DataType, Series};

pub async fn take_unchecked(datatype: DataType, mut src: &Vec<Chunk>, mut iter: &Vec<usize>) -> Vec<Chunk>{
    // let src_chunk_array_regular_ref = src.as_regular();
    // let iter_regular_ref = iter.as_regular();

    let total_len = iter.len();
    let mut new_serie = Series::new_from_name(datatype.clone(), "tmp", total_len);
    unsafe{new_serie.take_iter_unchecked(src, iter);}
    new_serie.into_raw().1
}