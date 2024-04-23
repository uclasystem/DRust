use std::borrow::Borrow;
use std::thread;

use good_memory_allocator::SpinLockedAllocator;
use num::integer::Roots;
use tokio::runtime::Runtime;

use crate::drust_std::primitives::DRust;
use crate::drust_std::thread::*;
use crate::drust_std::utils::*;

use super::single_strassen::*;
use super::*;

pub async fn single_strassen_mul(
    mut a: DVec<i32>,
    mut b: DVec<i32>,
    m0: usize,
) -> DVec<i32> {
    println!("single_strassen_mul");
    let matrix_a = Matrix::from_vec(a, m0);
    let matrix_b = Matrix::from_vec(b, m0);
    let result = strassen_mul(matrix_a, matrix_b).to_vec();
    println!("single_strassen_mul end");
    result
}

pub fn subadd(
    a: &DVec<i32>,
    b: &DVec<i32>,
    a_row_start: usize,
    a_col_start: usize,
    b_row_start: usize,
    b_col_start: usize,
    a_width: usize,
    b_width: usize,
    m: usize,
) -> DVec<i32> {
    let num = m;
    let mut results_vec = DVec::with_capacity(num * num);
    let a_ref = a.as_ref();
    let b_ref = b.as_ref();
    let results_mut = results_vec.as_mut();
    for i in 0..num {
        let a_start = (i + a_row_start) * a_width + a_col_start;
        let b_start = (i + b_row_start) * b_width + b_col_start;
        for j in 0..num {
            results_mut.push(a_ref[a_start + j] + b_ref[b_start + j]);
        }
    }
    results_vec
}

pub fn subsub(
    a: &DVec<i32>,
    b: &DVec<i32>,
    a_row_start: usize,
    a_col_start: usize,
    b_row_start: usize,
    b_col_start: usize,
    a_width: usize,
    b_width: usize,
    m: usize,
) -> DVec<i32> {
    let num = m;
    let mut results_vec = DVec::with_capacity(num * num);
    let a_ref = a.as_ref();
    let b_ref = b.as_ref();
    let results_mut = results_vec.as_mut();
    for i in 0..num {
        let a_start = (i + a_row_start) * a_width + a_col_start;
        let b_start = (i + b_row_start) * b_width + b_col_start;
        for j in 0..num {
            results_mut.push(a_ref[a_start + j] - b_ref[b_start + j]);
        }
    }
    results_vec
}

pub fn subcpy(
    a: &DVec<i32>,
    a_row_start: usize,
    a_col_start: usize,
    a_width: usize,
    m: usize,
) -> DVec<i32> {
    let num = m;
    let mut results_vec = DVec::with_capacity(num * num);
    let results_mut = results_vec.as_mut();
    let a_ref = a.as_ref();
    for i in 0..num {
        let a_start = (i + a_row_start) * a_width + a_col_start;
        results_mut.extend(&a_ref[a_start..a_start + num]);
    }
    results_vec
}

pub fn constitute(m11: DVec<i32>, m12: DVec<i32>, m21: DVec<i32>, m22: DVec<i32>) -> DVec<i32> {
    let m11_vec = m11.as_ref();
    let m12_vec = m12.as_ref();
    let m21_vec = m21.as_ref();
    let m22_vec = m22.as_ref();

    let m = m11_vec.len().sqrt();
    let mut results_vec = DVec::with_capacity(4 * m * m);
    let results_mut = results_vec.as_mut();
    for i in 0..m {
        let indx = i * m;
        results_mut.extend(&m11_vec[indx..indx + m]);
        results_mut.extend(&m12_vec[indx..indx + m]);
    }
    for i in 0..m {
        let indx = i * m;
        results_mut.extend(&m21_vec[indx..indx + m]);
        results_mut.extend(&m22_vec[indx..indx + m]);
    }
    results_vec
}

pub fn add(a: &mut DVec<i32>, b: &DVec<i32>) {
    let a_mut = a.as_mut();
    let b_ref = b.as_ref();
    for i in 0..a_mut.len() {
        a_mut[i] += b_ref[i];
    }
}

pub fn sub(a: &mut DVec<i32>, b: &DVec<i32>) {
    let a_mut = a.as_mut();
    let b_ref = b.as_ref();
    for i in 0..a_mut.len() {
        a_mut[i] -= b_ref[i];
    }
}

pub async fn par_strassen_mul(
    mut a: DVec<i32>,
    mut b: DVec<i32>,
    m0: usize,
    level: u32,
) -> DVec<i32> {
    let matrix_a = a.borrow();
    let matrix_b = b.borrow();
    let m = m0 / 2;
    /* Top left submatrix */
    let tl_row_start = 0;
    let tl_col_start = 0;

    /* Top right submatrix */
    let tr_row_start = 0;
    let tr_col_start = m;

    /* Bottom left submatrix */
    let bl_row_start = m;
    let bl_col_start = 0;

    /* Bottom right submatrix */
    let br_row_start = m;
    let br_col_start = m;

    let aa1 = subadd(
        matrix_a,
        matrix_a,
        tl_row_start,
        tl_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );
    let aa2 = subadd(
        matrix_a,
        matrix_a,
        bl_row_start,
        bl_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );
    let aa3 = subcpy(matrix_a, tl_row_start, tl_col_start, m0, m);
    let aa4 = subcpy(matrix_a, br_row_start, br_col_start, m0, m);
    let aa5 = subadd(
        matrix_a,
        matrix_a,
        tl_row_start,
        tl_col_start,
        tr_row_start,
        tr_col_start,
        m0,
        m0,
        m,
    );
    let aa6 = subsub(
        matrix_a,
        matrix_a,
        bl_row_start,
        bl_col_start,
        tl_row_start,
        tl_col_start,
        m0,
        m0,
        m,
    );
    let aa7 = subsub(
        matrix_a,
        matrix_a,
        tr_row_start,
        tr_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );

    let bb1 = subadd(
        matrix_b,
        matrix_b,
        tl_row_start,
        tl_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );
    let bb2 = subcpy(matrix_b, tl_row_start, tl_col_start, m0, m);
    let bb3 = subsub(
        matrix_b,
        matrix_b,
        tr_row_start,
        tr_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );
    let bb4 = subsub(
        matrix_b,
        matrix_b,
        bl_row_start,
        bl_col_start,
        tl_row_start,
        tl_col_start,
        m0,
        m0,
        m,
    );
    let bb5 = subcpy(matrix_b, br_row_start, br_col_start, m0, m);
    let bb6 = subadd(
        matrix_b,
        matrix_b,
        tl_row_start,
        tl_col_start,
        tr_row_start,
        tr_col_start,
        m0,
        m0,
        m,
    );
    let bb7 = subadd(
        matrix_b,
        matrix_b,
        bl_row_start,
        bl_col_start,
        br_row_start,
        br_col_start,
        m0,
        m0,
        m,
    );
    println!("level: {}", level);
    drop(matrix_a);
    println!("after drop matrix_a");
    drop(matrix_b);
    println!("after drop matrix_b");
    drop(a);
    println!("after drop a");
    drop(b);
    println!("after drop b");

    let thread_diverge: u32 = 7;

    // if thread_diverge.pow(level + 1) < THREADS_NUM as u32 {
    if level == 1 {
        let m1_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa1, bb1, m, level + 1))
        });
        let m2_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa2, bb2, m, level + 1))
        });
        let m3_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa3, bb3, m, level + 1))
        });
        let m4_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa4, bb4, m, level + 1))
        });
        let m5_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa5, bb5, m, level + 1))
        });
        let m6_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa6, bb6, m, level + 1))
        });
        let m7_handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa7, bb7, m, level + 1))
        });

        let mut m1 = m1_handle.join().unwrap();
        let mut m2 = m2_handle.join().unwrap();
        let mut m3 = m3_handle.join().unwrap();
        let mut m4 = m4_handle.join().unwrap();
        let mut m5 = m5_handle.join().unwrap();
        let mut m6 = m6_handle.join().unwrap();
        let mut m7 = m7_handle.join().unwrap();

        sub(&mut m7, &m5);
        add(&mut m7, &m4);
        add(&mut m7, &m1);
        add(&mut m5, &m3);
        add(&mut m4, &m2);
        sub(&mut m1, &m2);
        add(&mut m1, &m3);
        add(&mut m1, &m6);

        return constitute(m7, m5, m4, m1);
    }
    if level == 2
    /*&& THREADS_NUM >= 32*/
    {
        let branch_manager = unsafe { BRANCHES.as_ref().unwrap() };
        let thread_resource = branch_manager.get_resource(0);
        let m1_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa1, bb1, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m2_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa2, bb2, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m3_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa3, bb3, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m4_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa4, bb4, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m5_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa5, bb5, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m6_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa6, bb6, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });
        let thread_resource = branch_manager.get_resource(0);
        let m7_handle = thread::spawn(move || {
            let result = Runtime::new()
                .unwrap()
                .block_on(par_strassen_mul(aa7, bb7, m, level + 1));
            branch_manager.release_resource(thread_resource);
            result
        });

        let mut m1 = m1_handle.join().unwrap();
        let mut m2 = m2_handle.join().unwrap();
        let mut m3 = m3_handle.join().unwrap();
        let mut m4 = m4_handle.join().unwrap();
        let mut m5 = m5_handle.join().unwrap();
        let mut m6 = m6_handle.join().unwrap();
        let mut m7 = m7_handle.join().unwrap();

        sub(&mut m7, &m5);
        add(&mut m7, &m4);
        add(&mut m7, &m1);
        add(&mut m5, &m3);
        add(&mut m4, &m2);
        sub(&mut m1, &m2);
        add(&mut m1, &m3);
        add(&mut m1, &m6);

        return constitute(m7, m5, m4, m1);
    }

    // let aa_vec = vec![aa1, aa2, aa3, aa4, aa5, aa6, aa7];
    // let bb_vec = vec![bb1, bb2, bb3, bb4, bb5, bb6, bb7];
    // let mut handles_vec = vec![];
    let m1_handle = dspawn(single_strassen_mul(aa1, bb1, m));
    let m2_handle = dspawn(single_strassen_mul(aa2, bb2, m));
    let m3_handle = dspawn(single_strassen_mul(aa3, bb3, m));
    let m4_handle = dspawn(single_strassen_mul(aa4, bb4, m));
    let m5_handle = dspawn(single_strassen_mul(aa5, bb5, m));
    let m6_handle = dspawn(single_strassen_mul(aa6, bb6, m));
    let m7_handle = dspawn(single_strassen_mul(aa7, bb7, m));

    let mut m1: DVec<i32> = m1_handle.await.unwrap();
    let mut m2: DVec<i32> = m2_handle.await.unwrap();
    let mut m3: DVec<i32> = m3_handle.await.unwrap();
    let mut m4: DVec<i32> = m4_handle.await.unwrap();
    let mut m5: DVec<i32> = m5_handle.await.unwrap();
    let mut m6: DVec<i32> = m6_handle.await.unwrap();
    let mut m7: DVec<i32> = m7_handle.await.unwrap();

    sub(&mut m7, &m5);
    add(&mut m7, &m4);
    add(&mut m7, &m1);
    add(&mut m5, &m3);
    add(&mut m4, &m2);
    sub(&mut m1, &m2);
    add(&mut m1, &m3);
    add(&mut m1, &m6);

    constitute(m7, m5, m4, m1)
}
