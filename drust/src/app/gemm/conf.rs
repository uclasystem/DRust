pub const MATRIX_SIZE: usize = 32768;
pub const SINGLE_SIZE: usize = 16;

pub const THREADS_NUM: usize = 16 * crate::conf::NUM_SERVERS; // set to 2 threads per server because GAM will enter dead loop if more than 2 threads are used
pub const BRANCH_NUM: usize = 21;