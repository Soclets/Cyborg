// Either word would fit, so nothing says which you meant.
// trace: D149, D3
struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $w: wrapped = $r as wrapped;
    return void;
end fn.main
