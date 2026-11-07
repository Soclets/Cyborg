// trace: D158
struct pair:
    a: f64;
end struct pair

fn.main [] -> void:
    let $x: pair = pair[a = 0.5];
    let $y: pair = pair[a = 0.5];
    if $x == $y:
        return void;
    end if
    return void;
end fn.main
