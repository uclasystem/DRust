use std::sync::{atomic::AtomicBool, Condvar, Mutex};

use clap::Parser;

use super::NUM_SERVERS;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// server index
    #[arg(short, long)]
    serverid: u8,

    /// application name
    #[arg(short, long)]
    application: String,
}

pub fn get_args() -> (String, usize) {
    let args = Args::parse();
    (args.application, args.serverid as usize)
}

pub struct Resource {
    pub id: usize,
    pub manager: &'static ResourceManager,
}

impl Resource {
    pub fn release(self) {
        self.manager.release_resource(self);
    }
}

// Rewrite using conditional variables
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

    pub fn get_resource(&'static self, start_id: usize) -> Resource {
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
        Resource {
            id: rem,
            manager: self,
        }
    }

    pub fn release_resource(&self, res: Resource) {
        self.avail_resources[res.id].store(true, std::sync::atomic::Ordering::SeqCst);
        let mut lock = self.avail_num.lock().unwrap();
        *lock += 1;
        self.condvar.notify_all();
    }
}


pub static mut COMPUTES: Option<ResourceManager> = None;




pub struct SimpleResource {
    pub manager: &'static SimpleResourceManager,
}

impl SimpleResource {
    pub fn release(self) {
        self.manager.release_resource(self);
    }
}

// Rewrite using conditional variables
pub struct SimpleResourceManager {
    condvar: Condvar,
    resource_num: usize,
    avail_num: Mutex<usize>,
}

impl SimpleResourceManager {
    pub fn new(num: usize) -> Self {
        SimpleResourceManager {
            condvar: Condvar::new(),
            resource_num: num,
            avail_num: Mutex::new(num),
        }
    }

    pub fn get_resource(&'static self) -> SimpleResource {
        let mut lock = self.avail_num.lock().unwrap();
        while *lock == 0 {
            lock = self.condvar.wait(lock).unwrap();
        }
        *lock -= 1;
        SimpleResource {
            manager: self,
        }
    }

    pub fn release_resource(&self, res: SimpleResource) {
        let mut lock = self.avail_num.lock().unwrap();
        *lock += 1;
        self.condvar.notify_all();
    }
}


pub static mut SIMPLE_COMPUTES: Option<Vec<SimpleResourceManager>> = None;