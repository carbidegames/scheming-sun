extern crate sc_frontend;

use sc_frontend::Frontend;

fn main() {
    let frontend = Frontend::start();
    frontend.join();
}
