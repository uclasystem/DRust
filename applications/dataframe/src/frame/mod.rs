#[macro_use]
pub mod hashjoin;
pub mod groupby;
pub mod take;

use itertools::Itertools;
use serde::{ Serialize, Deserialize };
use rayon::{ prelude::*, vec };
use csv;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use super::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

type DfSeries = Series;
type DfColumns = Vec<DfSeries>;

pub struct DataFrame {
    columns: DfColumns,
}

impl DataFrame {
    /// Get the index of the column.
    fn name_to_idx(&self, name: &str) -> Result<usize, PolarsError> {
        let mut idx = 0;
        for column in &self.columns {
            if column.name() == name {
                break;
            }
            idx += 1;
        }
        if idx == self.columns.len() {
            Err(PolarsError::NotFound)
        } else {
            Ok(idx)
        }
    }

    /// Create a DataFrame from a Vector of Series.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// let s0 = Series::new("days", [0, 1, 2].as_ref());
    /// let s1 = Series::new("temp", [22.1, 19.9, 7.].as_ref());
    /// let df = DataFrame::new(vec![s0, s1]).unwrap();
    /// ```
    pub fn new(columns: Vec<Series>) -> Result<Self, PolarsError> {
        Ok(DataFrame { columns })
    }

    /// Get a reference to the DataFrame columns.
    pub fn get_columns(&self) -> &DfColumns {
        &self.columns
    }

    /// Get the column labels of the DataFrame.
    pub fn columns(&self) -> Vec<&str> {
        self.columns
            .iter()
            .map(|s| s.name())
            .collect()
    }

    /// The number of chunks per column
    pub fn n_chunks(&self) -> Result<usize, PolarsError> {
        Ok(self.columns.get(0).ok_or(PolarsError::NoData)?.n_chunks())
    }

    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Remove column by name
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// fn drop_column(df: &mut DataFrame, name: &str) -> Result<Series> {
    ///     df.drop_in_place(name)
    /// }
    /// ```
    pub fn drop_in_place(&mut self, name: &str) -> Result<DfSeries, PolarsError> {
        let idx = self.name_to_idx(name)?;
        let result = Ok(self.columns.remove(idx));
        result
    }

    /// Select a series by index.
    pub fn select_idx(&self, idx: usize) -> Option<&Series> {
        self.columns.get(idx)
    }

    /// Force select.
    pub fn f_select_idx(&self, idx: usize) -> &Series {
        self.select_idx(idx).expect("out of bounds")
    }

    /// Select a mutable series by index.
    pub fn select_idx_mut(&mut self, idx: usize) -> Option<&mut Series> {
        self.columns.get_mut(idx)
    }

    /// Force select.
    pub fn f_select_idx_mut(&mut self, idx: usize) -> &mut Series {
        self.select_idx_mut(idx).expect("out of bounds")
    }

    /// Get column index of a series by name.
    pub fn find_idx_by_name(&self, name: &str) -> Option<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter(|(_idx, field)| field.name() == name)
            .map(|(idx, _)| idx)
            .next()
    }

    /// Select a single column by name.
    pub fn column(&self, name: &str) -> Option<&Series> {
        let opt_idx = self.find_idx_by_name(name);

        match opt_idx {
            Some(idx) => self.select_idx(idx),
            None => None,
        }
    }

    /// Force select a single column.
    pub fn f_column(&self, name: &str) -> &Series {
        self.column(name).expect(&format!("name {} does not exist on dataframe", name))
    }

    /// Select a mutable series by name.
    pub fn select_mut(&mut self, name: &str) -> Option<&mut Series> {
        let opt_idx = self.find_idx_by_name(name);

        match opt_idx {
            Some(idx) => self.select_idx_mut(idx),
            None => None,
        }
    }

    /// Force select.
    pub fn f_select_mut(&mut self, name: &str) -> &mut Series {
        self.select_mut(name).expect(&format!("name {} does not exist on dataframe", name))
    }

    
    pub async fn get(
        &mut self,
        idx: usize
    ) -> Option<Vec<AnyType>> {
        let mut row = Vec::new();
        for column in &self.columns {
            let elem = column.get(idx).await;
            row.push(elem);
        }
        Some(row)
    }

    pub async unsafe fn take_iter_unchecked(&self, iter: &Vec<usize>, capacity: usize, drop_names: Vec<String>) -> Self
    {
        println!("num threads: {}", rayon::current_num_threads());
        std::thread::scope(|c| {
            let mut vec_handles = vec![];
            for s in &self.columns {
                if !drop_names.contains(&s.name().to_owned()) {
                    let datatype = s.dtype().clone();
                    let src_serie_name = s.name().to_string();
                    let src_ref = s.get_ref();
                    let iter_ref = iter.as_ref();
                    let printname = src_serie_name.clone();
                    println!("In send taking iter unchecked request: {}", printname);
                    let handle = c.spawn(|| {Runtime::new().unwrap().block_on(take::take_unchecked(datatype, src_ref, iter_ref))});
                    vec_handles.push((handle, src_serie_name, s.dtype().clone()));
                }
            }
            let mut results = vec![];
            for f in vec_handles {
                let ca = f.0.join().unwrap();
                let serie_name = f.1;
                let datatype = f.2;
                let field = Field::new(&serie_name, datatype, true);
                let mut serie = Series::from_raw(field, ca);
                results.push(serie);
            }
            println!("Finished whole dataframe taking iter unchecked");
            DataFrame::new(results).unwrap()
        })
    }
}