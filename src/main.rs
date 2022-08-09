//make use of https://crates.io/crates/image to decode main image formats
//make use of https://github.com/pedrocr/rawloader/ for decoding of RAW images
//make use of https://docs.rs/druid/0.7.0/druid/widget/struct.Image.html the druid image widget for displaying images

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::*;
use ogl33::*;

fn main() {
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();
    #[cfg(target_os = "macos")]
    {
        sdl
        .gl_set_attribute(SdlGlAttr::Flags, ContextFlag::ForwardCompatible)
        .unwrap();
    }

    let win = sdl
        .create_gl_window(
            "Hello Window",
            WindowPosition::Centered,
            800,
            600,
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    
    unsafe {
        load_gl_with(|f_name| win.get_proc_address(f_name));
    }

    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
            Event::Quit(_) => break 'main_loop,
            _ => (),
            }
        }
        // now the events are clear
    
        // here's where we could change the world state and draw.
    }
}
