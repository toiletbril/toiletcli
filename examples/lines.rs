use toiletcli::ansi::*;

fn main() {
    println!("This is a 'word' that will be replaced!{}bird", Cursor::Column(12));
    // This is a 'bird' that will be replaced!

    println!("This is a '{}dog' that will be replaced too!{}cat", Cursor::Save, Cursor::Restore);
    // This is a 'cat' that will be replaced too!

    print!("{}", System::SetTitle("hello"));

    loop {}
}
