// A string's `len.max` is in bytes, since UTF-8 characters vary in size.
// trace: D326
fn.main [none] -> void:
    let $name: str[len.max: 6 items] = "Tankun";
    return void;
end fn.main
