// trace: D160, D134

type.new temperature set_to f64.stable;

fn.main [none] returns void:
    let $a: temperature set_to 1.5;
    let $b: temperature set_to 2.5;
    if $a == $b:
        return void;
    end if
    return void;
end fn.main
