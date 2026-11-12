// .%reveal() only lives in a shield, whose reason says why.
// trace: D342
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $p: u32 = $key.%reveal();
    return void;
end fn.main
