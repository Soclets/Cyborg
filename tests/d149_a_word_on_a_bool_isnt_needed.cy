// trace: D149, D123

type.new flag = bool;

fn.main [none] returns void:
    let $b: bool = true;
    let $f: flag = $b as flag;
    let $g: flag = Copy $b as flag;
    return void;
end fn.main
