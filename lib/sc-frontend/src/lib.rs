extern crate cgmath;
#[macro_use] extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate sc_input_data;

mod framecounter;
mod frontend;
mod runtime;
mod teapot;

mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_vs.glsl")} }
mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_fs.glsl")} }

use sc_input_data::Button;

pub use frontend::Frontend;

pub enum FrontendEvent {
    Closed,
    ButtonState(Button, bool),
    __DoNotMatch,
}

// TODO: Doesn't need to be publically exposed
pub enum FrontendCommand {
    Stop,
}
