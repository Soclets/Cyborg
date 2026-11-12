// A secret goes where a plain value is expected only through .%reveal().
// trace: D340, D342
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $p: u32 = $key;
    return void;
end fn.main
