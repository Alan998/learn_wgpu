// Arc: Atomic Reference Counted (similar to a smart pointer)
use std::sync::Arc;

// winit is a cross-platform windowing and event loop library
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
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
        Ok(Self { window })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}

    pub fn render(&mut self) {
        // make the window draw another frame as soon as possible.
        // winit only draws one frame unless the window is resized or receiving a request_redraw
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
    // resumed method is called by winit when the window becomes "resumed" or "active"
    // resumed method is usually used for:
    // 1. create the application window if it does not exist
    // 2. initialize the application's state, including the wgpu rendering context

    // self is a mutable reference to App to modify it's state
    // event_loop provides access to currently active winit event loop
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        // initialize a mutable window_attributes with default values
        // WindowAttributes define properties of the window you want to create (e.g., title,
        // size...)
        let mut window_attributes = Window::default_attributes();

        // wasm specific setup
        #[cfg(target_arch = "wasm32")]
        {
            // import JsCast trait for safe casting betwteen Javascript types
            use wasm_bindgen::JsCast;
            // import WindowAttributesExtWebSys trait for wasm-specific methods
            use winit::platform::web::WindowAttributesExtWebSys;

            // defines a  constant for the ID of theh HTML <canvas> element that the wgpu app will
            // draw onto
            const CANVAS_ID: &str = "canvas";

            // web_sys::window() is a function from the web-sys crate that
            // gets a reference to the browser's global Window object.
            //
            // .unwrap_throw() is a wasm-bindgen utility function.
            // If the Option returned by window() is None (meaning no window context is available,
            // which shouldn't happen in a browser), it will immediately throw a JavaScript error,
            // halting execution. It's similar to Rust's unwrap(), but for wasm error handling.
            let window = wgpu::web_sys::window().unwrap_throw();

            // gets a reference to the browser's `Document` object from the `Window`
            let document = window.document().unwrap_throw();
            // finds the HTML element with the ID "canvas" in the document
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            // This is an unsafe method from wasm-bindgen::JsCast that performs a type assertion.
            // It casts the generic Element (returned by get_element_by_id) into a specific HtmlCanvasElement.
            // This is necessary because winit's with_canvas expects a typed HtmlCanvasElement.
            // (commonly used and often safe in practice when you know the element type)
            let html_canvas_element = canvas.unchecked_into();

            // This is the critical part for WASM.
            // It modifies the window_attributes to tell `winit` that
            // instead of creating a new OS window,
            // it should use the existing HTML <canvas> element just found.
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        // event_loop.create_window is the init function for creating a window
        // wraps the created window in an Arc for shared ownership, or panics if window creating
        // fails
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // this block only runs on native desktop builds
        #[cfg(not(target_arch = "wasm32"))]
        {
            // pollster::block_on is a utility funciton that takes an `async Future` and
            // runs it to completion on the current thread, blocking until the `Future` finishes
            //
            // Why pollster::block_on here?
            // On native platforms, the resumed event itself is often called from a synchroonous
            // context (the main event loop thread). Since `State::new()` is async, it needs a
            // way to execute that async code in a blocking manner.
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            //
            // take() replaces the Some(proxy) with None, ensuring that this initialization logic runs
            // only once
            if let Some(proxy) = self.proxy.take() {
                // wasm_bindgen_futures::spawn_local is a crucial function for running async Rust
                // code in a web browser.
                // It takes an async block (a Future) and schedules it to run on the browser's event
                // loop (the main JavaScript thread). It does not block the current thread.
                //
                // send_event() sends the newly initialized State instance as a custom event back to
                // the winit event loop.
                // This is how you communicate the result of the asynchronous State creation back to
                // the main App logic.
                //
                // assert!(...).is_ok() Asserts that sending the event was successful.
                // send_event can fail if the event loop has already been closed.
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
                                    .await // await pauses the execution of this async move block until State::new completes
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    // user_event just serves as a landing point for our `State` future.
    // `resumed` is not async so we need to offload the future and send the results somewhere
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        // This is where proxy.send_event() ends up
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.render();
            }
            // The curly braces {} allow for destructuring the KeyboardInput variant.
            // This means its internal fields can be pulled out.
            WindowEvent::KeyboardInput {
                event: // This destructures the inner `KeyEvent` struct
                KeyEvent {
                    // Why PhysicalKey::Code?
                    // For game controls or system commands (like quitting with Escape),
                    // using PhysicalKey::Code is often preferred because it's consistent
                    // across different keyboard layouts.
                    physical_key: PhysicalKey::Code(code), // Extracts the physical key code (e.g., A, Escape)
                    state, // Extracts the key state (Pressed or Released)
                    .. // Ignores other fields of KeyEvent (e.g., logical_key, text)
                },
                .. // Ignores other fields of WindowEvent::KeyboardInput
            } => match (code, state.is_pressed()) { // 
                (KeyCode::Escape, true) => event_loop.exit(), // exit if ESC is pressed
                _ => {} // do nothing if other keys are pressed
            },
            _ => {}
        }
    }
}

// create a run function to run the code
// This function sets up the logger as well as creates the event_loop and our app and then
// runs our app to completion
pub fn run() -> anyhow::Result<()> {
    // initialize logging
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    // Create the winit EventLoop
    // This mechanism dispatches events (user input, window events...) to the application.
    // .with_user_event() allows sending custom events later (used in WASM setup)
    // .build()? creates the event loop, propagating any build errors
    let event_loop = EventLoop::with_user_event().build()?;

    // create main App struct
    // The event_loop parameter is conditionally passed for WASM targets
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );

    // start the winit event loop, handing control to your App
    event_loop.run_app(&mut app)?;

    // If the event loop exits successfully, return Ok(())
    // (): This is the "unit type" in Rust, essentially meaning "nothing" or "no specific value."
    // When a function returns Ok(()), it signifies success without returning any particular data.
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
// Set up console_error_panic_hook so we can see the code panic information in the browser
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
