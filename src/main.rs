use gap_buffer::GapBuffer;

fn main() {
    let data = (0..)
        .take(7)
        .map(|n| format!("h{}", n))
        .collect::<Vec<String>>();

    let mut gb = GapBuffer::new(data);
    println!("{gb}\n");

    gb.set_position(4);
    gb.remove();
    gb.set_position(6);
    gb.remove();
    gb.set_position(0);
    gb.remove();
    println!("{gb}\n");

    gb.insert("h0".to_string());

    gb.set_position(4);
    gb.insert("h4".to_string());

    gb.set_position(7);
    gb.insert("h7".to_string());
    println!("{gb}\n");

    let addition = (8..).take(5).map(|n| format!("h{}", n));
    gb.insert_iter(addition);
    println!("{gb}\n");
}
