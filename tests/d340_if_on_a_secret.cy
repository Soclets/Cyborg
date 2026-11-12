// The path an if takes would leak the secret.
// trace: D340
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    if $key == 0:
        return void;
    end if
    return void;
end fn.main
