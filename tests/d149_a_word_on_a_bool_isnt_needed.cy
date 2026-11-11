// trace: D149, D123

type.new flag set_to bool;

fn.main [none] returns void:
    let $b: bool set_to true;
    let $f: flag set_to $b as flag;
    let $g: flag set_to Copy $b as flag;
    return void;
end fn.main
