// Temporary diagnostic: report pointer location, then define a standard arrow
// cursor on the root window so we can see if the root cursor was blanked.
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt};

fn main() {
    let (conn, screen) = x11rb::connect(None).expect("connect");
    let root = conn.setup().roots[screen].root;

    let ptr = conn
        .query_pointer(root)
        .expect("query_pointer")
        .reply()
        .expect("query_pointer reply");
    println!(
        "pointer at root coords ({}, {}), child window: 0x{:x}",
        ptr.root_x, ptr.root_y, ptr.child
    );

    // standard arrow: built-in "cursor" font (id 0), glyph 0 (X_cursor)
    let cursor = conn.generate_id().expect("id");
    conn.create_glyph_cursor(cursor, 0, x11rb::NONE, 0, 0, 0, 0, 0, 0, 0, 0)
        .expect("create_glyph_cursor");
    conn.change_window_attributes(root, &ChangeWindowAttributesAux::new().cursor(cursor))
        .expect("change_window_attributes");
    conn.flush().expect("flush");
    println!("defined standard arrow cursor on root; sleeping 120s to observe...");
    std::thread::sleep(std::time::Duration::from_secs(600));
}
