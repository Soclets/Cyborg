// trace: D149

type.new title set_to str;

fn.main [none] returns void:
    let $s: str set_to "a";
    let $t: title set_to $s as title;
    return void;
end fn.main
