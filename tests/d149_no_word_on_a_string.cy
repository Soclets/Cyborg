// trace: D149

type.new title = str[len.max: 32 bytes];

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    let $t: title = $s as title;
    return void;
end fn.main
