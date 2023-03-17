use super::TextBuffer;

#[test]
fn insert_line_works() {
    let mut buffer = TextBuffer::from("");
    buffer.insert_line(0, "hello");
    buffer.insert_line(1, "");
    buffer.insert_line(2, "world");

    assert_eq!(buffer.data, "helloworld");
    assert_eq!(buffer.line_sizes, [5, 0, 5]);
}

#[test]
fn remove_line_works() {
    let mut buffer = TextBuffer::from("hello\n\nworld");

    buffer.remove_line(1);
    assert_eq!(buffer.data, "helloworld");
    assert_eq!(buffer.line_sizes, [5, 5]);

    buffer.remove_line(0);
    assert_eq!(buffer.data, "world");
    assert_eq!(buffer.line_sizes, [5]);
}
