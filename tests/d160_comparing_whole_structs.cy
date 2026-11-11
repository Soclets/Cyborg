// trace: D160

struct pair:
    a: f64.stable;
end struct pair

fn.main [none] returns void:
    let $x: pair set_to pair[a set_to 0.5];
    let $y: pair set_to pair[a set_to 0.5];
    if $x == $y:
        return void;
    end if
    return void;
end fn.main
