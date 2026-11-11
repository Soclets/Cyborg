// trace: D149, D2

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 1];
    let $yes: bool set_to true;
    if $yes:
        let $w: wrapped set_to Give $r as wrapped;
    end if
    let $v: wrapped set_to Copy $r as wrapped;
    return void;
end fn.main
