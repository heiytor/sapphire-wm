use xcb::x;

fn main() {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let window: x::Window = conn.generate_id();

    _ = conn.send_request(&x::CreateWindow{
        depth: x::COPY_FROM_PARENT as u8,
        wid: window,
        parent: screen.root(),
        x: 0,
        y: 0,
        width: 150,
        height: 150,
        border_width: 10,
        class: x::WindowClass::InputOutput,
        visual: screen.root_visual(),
        value_list: &[
               x::Cw::BackPixel(screen.black_pixel()),
               x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
        ],
    });

    _ = conn.send_request(&x::MapWindow{ window });

    _ = conn.send_request(&x::ChangeProperty{
        window,
        mode: x::PropMode::Replace,
        r#type: x::ATOM_STRING,
        data: "window".as_bytes(),
        property: x::ATOM_WM_NAME,
    });

    conn.flush().unwrap();
    
    loop {
    
    }
}
