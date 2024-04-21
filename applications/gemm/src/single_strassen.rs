use super::conf::*;

pub struct Matrix {
    pub row: usize,
    pub col: usize,
    pub elements: Vec<i32>,
}

impl Matrix {
    pub fn new(row_size: usize, element_value: i32) -> Self {
        let elements: Vec<i32> = vec![element_value; row_size * row_size];
        Matrix {
            row: row_size,
            col: row_size,
            elements,
        }
    }

    pub fn from_vec(elements: Vec<i32>, row_size: usize) -> Self {
        let mut matrix_elements = Vec::with_capacity(row_size * row_size);
        for i in 0..row_size {
            for j in 0..row_size {
                matrix_elements.push(elements[i * row_size + j]);
            }
        }
        Matrix {
            row: row_size,
            col: row_size,
            elements: matrix_elements,
        }
    }

    pub fn to_vec(&self) -> Vec<i32> {
        let mut elements = Vec::with_capacity(self.row * self.col);
        let elements_mut: &mut Vec<i32> = elements.as_mut();
        elements_mut.extend(&self.elements[..]);
        elements
    }

    pub fn new_uninit(segment_num: usize) -> Self {
        let mut elements: Vec<i32> = Vec::with_capacity(segment_num * segment_num);
        unsafe {
            elements.set_len(segment_num * segment_num);
        }
        Matrix {
            row: segment_num,
            col: segment_num,
            elements,
        }
    }

    /**
     * Adds the contents of `b` to this Matrix, and returns self. Panics if `b` is not the same size as this Matrix.
     */
    pub fn add(&mut self, b: &Matrix) -> &mut Matrix {
        if self.row != b.row || self.col != b.col {
            panic!("Matrix size not match");
        }
        for i in 0..self.elements.len() {
            self.elements[i] += b.elements[i];
        }
        self
    }

    /**
     * Subtracts the contents of `b` from this Matrix, and returns self. Panics if `b` is not the same size as this Matrix.
     */
    pub fn sub(&mut self, b: &Matrix) -> &mut Matrix {
        if self.row != b.row || self.col != b.col {
            panic!("Matrix size not match");
        }
        for i in 0..self.elements.len() {
            self.elements[i] -= b.elements[i];
        }
        self
    }

    pub fn subadd(&self, row0: usize, col0: usize, row1: usize, col1: usize, width: usize) -> Matrix {
        let mut results = Matrix::new_uninit(width);
        for i in 0..width {
            let start0 = (i + row0) * self.col + col0;
            let start1 = (i + row1) * self.col + col1;
            for j in 0..width {
                results.elements[i * width + j] = self.elements[start0 + j] + self.elements[start1 + j];
            }
        }
        results
    }

    pub fn subsub(&self, row0: usize, col0: usize, row1: usize, col1: usize, width: usize) -> Matrix {
        let mut results = Matrix::new_uninit(width);
        for i in 0..width {
            let start0 = (i + row0) * self.col + col0;
            let start1 = (i + row1) * self.col + col1;
            for j in 0..width {
                results.elements[i * width + j] = self.elements[start0 + j] - self.elements[start1 + j];
            }
        }
        results
    }

    pub fn subcpy(&self, row0: usize, col0: usize, width: usize) -> Matrix {
        let mut results = Matrix::new_uninit(width);
        for i in 0..width {
            let start0 = (i + row0) * self.col + col0;
            for j in 0..width {
                results.elements[i * width + j] = self.elements[start0 + j];
            }
        }
        results
    }

    pub fn constitute(m11: Matrix, m12: Matrix, m21: Matrix, m22: Matrix) -> Matrix {
        let m0 = m11.row;
        let mut result = Matrix::new_uninit(m0 * 2);
        for i in 0..m0 {
            for j in 0..m0 {
                result.elements[i * m0 * 2 + j] = m11.elements[i * m0 + j];
                result.elements[i * m0 * 2 + j + m0] = m12.elements[i * m0 + j];
                result.elements[(i + m0) * m0 * 2 + j] = m21.elements[i * m0 + j];
                result.elements[(i + m0) * m0 * 2 + j + m0] = m22.elements[i * m0 + j];
            }
        }
        result
    }
}


pub fn mul_simple(a: Matrix, b: Matrix, m0: usize) -> Matrix {
    let mut result = Matrix::new(m0, 0);
    for i in 0..m0 {
        for j in 0..m0 {
            let mut sum = 0;
            for k in 0..m0 {
                sum += a.elements[i * m0 + k] * b.elements[k * m0 + j];
            }
            result.elements[i * m0 + j] = sum;
        }
    }
    result
}


pub fn strassen_mul(a: Matrix, b: Matrix) -> Matrix {
    let m0 = a.row;
    if m0 == SINGLE_SIZE {
        return mul_simple(a, b, m0);
    }

    let matrix_a = &a;
    let matrix_b = &b;
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

    /* Vectors for 7 submatrices of `a` */
    let aa1 = matrix_a.subadd(tl_row_start, tl_col_start, br_row_start, br_col_start, m);
    let aa2 = matrix_a.subadd(bl_row_start, bl_col_start, br_row_start, br_col_start, m);
    let aa3 = matrix_a.subcpy(tl_row_start, tl_col_start, m);
    let aa4 = matrix_a.subcpy(br_row_start, br_col_start, m);
    let aa5 = matrix_a.subadd(tl_row_start, tl_col_start, tr_row_start, tr_col_start, m);
    let aa6 = matrix_a.subsub(bl_row_start, bl_col_start, tl_row_start, tl_col_start, m);
    let aa7 = matrix_a.subsub(tr_row_start, tr_col_start, br_row_start, br_col_start, m);


    /* Vectors for 7 submatrices of `b` */
    let bb1 = matrix_b.subadd(tl_row_start, tl_col_start, br_row_start, br_col_start, m);
    let bb2 = matrix_b.subcpy(tl_row_start, tl_col_start, m);
    let bb3 = matrix_b.subsub(tr_row_start, tr_col_start, br_row_start, br_col_start, m);
    let bb4 = matrix_b.subsub(bl_row_start, bl_col_start, tl_row_start, tl_col_start, m);
    let bb5 = matrix_b.subcpy(br_row_start, br_col_start, m);
    let bb6 = matrix_b.subadd(tl_row_start, tl_col_start, tr_row_start, tr_col_start, m);
    let bb7 = matrix_b.subadd(bl_row_start, bl_col_start, br_row_start, br_col_start, m);


    
    let mut m1 = strassen_mul(aa1, bb1);
    let m2 = strassen_mul(aa2, bb2);
    let m3 = strassen_mul(aa3, bb3);
    let mut m4 = strassen_mul(aa4, bb4);
    let mut m5 = strassen_mul(aa5, bb5);
    let m6 = strassen_mul(aa6, bb6);
    let mut m7 = strassen_mul(aa7, bb7);

    m7.sub(&m5).add(&m4).add(&m1);
    m5.add(&m3);
    m4.add(&m2);
    m1.sub(&m2).add(&m3).add(&m6);
    Matrix::constitute(m7, m5, m4, m1)
}
