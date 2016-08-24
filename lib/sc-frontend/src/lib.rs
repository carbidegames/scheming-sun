extern crate cgmath;
#[macro_use] extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

mod runtime;
mod teapot;

mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_vs.glsl")} }
mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_fs.glsl")} }

use std::thread::{self, JoinHandle};

pub struct Frontend {
    handle: JoinHandle<()>
}

impl Frontend {
    pub fn start() -> Self {
        let handle = thread::spawn(|| {
            runtime::frontend_runtime();
        });

        Frontend {
            handle: handle
        }
    }

    pub fn join(self) {
        self.handle.join().unwrap();
    }
}
