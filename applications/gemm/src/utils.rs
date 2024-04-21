use std::sync::{atomic::AtomicBool, Condvar, Mutex};

pub static mut BRANCHES: Option<ResourceManager> = None;
pub static mut COMPUTES: Option<ResourceManager> = None;

pub struct ResourceManager {
    condvar: Condvar,
    resource_num: usize,
    avail_num: Mutex<usize>,
    avail_resources: Vec<AtomicBool>,
}
impl ResourceManager {
    pub fn new(num: usize) -> Self {
        let mut avail_resources = Vec::with_capacity(num);
        for _ in 0..num {
            avail_resources.push(AtomicBool::new(true));
        }
        ResourceManager {
            condvar: Condvar::new(),
            resource_num: num,
            avail_num: Mutex::new(num),
            avail_resources,
        }
    }

    pub fn get_resource(&self, start_id: usize) -> usize {
        let mut lock = self.avail_num.lock().unwrap();
        while *lock == 0 {
            lock = self.condvar.wait(lock).unwrap();
        }
        *lock -= 1;
        let mut rem = start_id % self.resource_num;
        let cycle = rem;
        loop {
            if self.avail_resources[rem]
                .compare_exchange(
                    true,
                    false,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                break;
            }
            rem = (rem + 1) % self.resource_num;
        }
        rem
    }

    pub fn release_resource(&self, res: usize) {
        self.avail_resources[res].store(true, std::sync::atomic::Ordering::SeqCst);
        let mut lock = self.avail_num.lock().unwrap();
        *lock += 1;
        self.condvar.notify_all();
    }
}
