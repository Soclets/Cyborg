// A based literal is checked against its type like any other.
// trace: D374
fn.main [none] -> void:
    let $byte: u8 = hex 1_00;
    return void;
end fn.main
