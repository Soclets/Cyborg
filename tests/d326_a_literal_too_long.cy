// A string's `len.max` is in bytes, and a literal that can't fit is a compile error.
// trace: D326, D20
fn.main [none] -> void:
    let $name: str[len.max: 4 bytes] = "Tankun";
    return void;
end fn.main
