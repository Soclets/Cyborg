// trace: D149, D2
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [] -> void:
    let $r: reading = reading[celsius = 1];
    let $yes: bool = true;
    if $yes:
        let $w: wrapped = Give $r as wrapped;
    end if
    let $v: wrapped = Copy $r as wrapped;
    return void;
end fn.main
