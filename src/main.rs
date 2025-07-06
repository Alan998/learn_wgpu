use learn_wgpu::run;

fn main() {
    unsafe {
        std::env::set_var("WAYLAND_DISPLAY", ""); // Force X11 on Linux
    }
    run().unwrap();
}
