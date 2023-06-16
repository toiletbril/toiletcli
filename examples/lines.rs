use toiletcli::lines::*;

fn main() {
    println!("This is a 'word' that will be replaced!{}bird", Cursor::Position(12));
    // This is a 'bird' that will be replaced!

    println!("This is a '{}dog' that will be replaced too!{}cat", Cursor::Save, Cursor::Restore);
    // This is a 'cat' that will be replaced too!
}