use std::{
    fs::File,
    io::{Write, self},
    time::{Duration, SystemTime},
};

use tarpc::context;
use tokio::task::JoinHandle;

use super::super::series::input::read_series;

use super::super::prelude::*;

#[derive(Debug)]
pub enum DSize {
    Small,
    Medium,
    Large,
    Huge,
}


pub async fn read_csv_from_file(
    file_name: &str,
    types: Vec<DataType>,
    line_cnt: usize,
) -> Result<DataFrame, PolarsError> {
    let full_path = format!(
        "{}/DRust_home/dataset/dataframe/my_{}", dirs::home_dir().unwrap().display(),
        file_name
    );
    let mut reader = csv::Reader::from_path(full_path).unwrap();
    let headers = reader.headers().unwrap();
    if headers.len() != types.len() {
        return Err(PolarsError::DataTypeMisMatch);
    }
    let mut series: Vec<Series> = Vec::new();
    let mut jobs = vec![];
    let mut index: usize = 0;
    // let results: CHashMap<String, u64> = CHashMap::new();
    // let results_ref = Arc::new(results);

    for dtype in &types {
        let f = file_name.to_string();
        let datatype = dtype.clone();
        let line_count = line_cnt;
        let series_id = index;

        let handle: JoinHandle<Vec<Chunk>> = tokio::spawn(read_series(datatype, series_id, line_count));
        jobs.push(handle);
        index += 1;
    }

    index = 0;
    for job in jobs {
        println!("awaiting job {}", index);
        let w = job.await.unwrap();
        println!("job {} done", index);
        println!("len: {}", w.len());
        let datatype = types[index].clone();
        let name = &headers[index];
        let field = Field::new(name, datatype, true);
        let mut serie = Series::from_raw(field, w);
        index += 1;
        series.push(serie);
    }
    DataFrame::new(series)
}

pub async fn print(df: &mut DataFrame) {
    for id in 0..5 {
        let a = df.get(id).await.expect("Should have");
        for value in a {
            match value {
                AnyType::Boolean(v) => print!("bool: {} ", v),
                AnyType::UInt8(v) => print!("u8: {} ", v),
                AnyType::UInt16(v) => print!("u16: {} ", v),
                AnyType::UInt32(v) => print!("u32: {} ", v),
                AnyType::UInt64(v) => print!("u64: {} ", v),
                AnyType::Int8(v) => print!("i8: {} ", v),
                AnyType::Int16(v) => print!("i16: {} ", v),
                AnyType::Int32(v) => print!("i32: {} ", v),
                AnyType::Int64(v) => print!("i64: {} ", v),
                AnyType::Float32(v) => print!("f32: {} ", v),
                AnyType::Float64(v) => print!("f64: {} ", v),
                _ => print!("other"),
            };
        }
        println!();
    }
}

pub fn print_time(time: u128, accum_time: u128, wrt_file: &mut File, dataset_size: &DSize) {
    writeln!(
        wrt_file,
        "query {:?}, {}ms, accum: {} ms",
        dataset_size, time, accum_time
    )
    .expect("could not write");
    println!("query: {} ms, accum: {} ms", time, accum_time);
}
