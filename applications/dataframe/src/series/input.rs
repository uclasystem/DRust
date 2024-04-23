use crate::datatype_to_anytype;

use super::{super::{prelude::*, self_arrow::DataType, DATASET_NAME}, chunk::Chunk, AnyType, Series};

pub async fn read_series(datatype: DataType, series_id: usize, line_count: usize) -> Vec<Chunk> {
    let full_path = format!("{}/DRust_home/dataset/dataframe/my_{}", dirs::home_dir().unwrap().display(), DATASET_NAME);
    let mut reader = csv::Reader::from_path(full_path).unwrap();
    let headers = reader.headers().unwrap();
    if headers.len() < series_id {
        panic!("Series id is out of bound");
    }
    let name = &headers[series_id];
    let mut serie = Series::new_from_name(datatype.clone(), name, line_count);
    let mut line_id = 0;
    for result in reader.records() {
        let record = result.unwrap();
        let v: AnyType = datatype_to_anytype!(datatype, record[series_id].parse().unwrap());
        serie.push_item(v, line_id);
        line_id += 1;
    }
    let (_, result) = serie.into_raw();
    result
}