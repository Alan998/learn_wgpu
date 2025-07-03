// Arc: Atomic Reference Counted (similar to a smart pointer)
use std::sync::Arc;

// winit is a cross-platform windowing and event loop library
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{
        ActiveEventLoop,
        EventLoop
    },
    keyboard::{KeyCode, PhysicalKey},
    window:::Window
};

// conditional compilation attribute
// the line below will only be included in the compiled code if the target architecture is wasm32
#[cfg(target_arch = "wasm32")]
// wasm_bindgen is a library for interactions between Rust and Javascript
// This library can expose Rust functions to Javascript, manipulate DOM...
use wasm_bindgen::prelude::*;

// This will store the state of our game
pub struct State {
    // Different parts of the application need to access the Window object,
    // Arc ensures that the Window is only dropped when all Arc pointers are out of scope
    window: Arc<Window>, 
}


impl State {
    // Why use async?
    // It is common for graphics initialization to involve asynchronous operations.
    // For instance, requesting an Adapter or Device from wgpu typically uses async
    // because these operations might wait for GPU drivers or the OS
    //
    // anhyhow::Result<T> is a popular and convenient type for error handling provided
    // by the `anyhow` crate
    // anyhow::Result<T> is a specialized Result where the error type E is automatically
    // handled by `anyhow` to be a dynamic error type (anyhow::Error).
    // It allow for easy propaagation by using ? operator.
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        // 'Self' here refers to the State struct itself.
        // So, this is returning an instance of State
        OK(Self {window})
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
    }

    pub fn render(&mut self) {
        self.window.request_redraw();
    }
}

// App struct tells winit how to use the State struct
pub struct App {
    #[cfg(target_arch = "wasm32")]
    // proxy is only needed on the web since creating WGPU resources is a async process
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,

    // state stores the State struct as an Option
    // Option is used since State::new() needs a window but window can't be created
    // until the application get to the `Resume` state
    state: Option<State>,
}


impl App {
    // For WebAssembly builds:
    // The new function will have a parameter named event_loop of type &EventLoop<State>.
    // This event_loop is necessary on the web to create the EventLoopProxy.
    // For Native builds:
    // The new function will not have an event_loop parameter at all.
    // Its signature will effectively be pub fn new() -> Self.
    // The compiler completely omits parameter event_loop for non-WASM builds.
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

// implement ApplicationHandler trait for App
// This allows App to get application events such as key press, mouse movements and various lifecycle events.
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
        }
    }

}
