// trace: D149
@config(default);

type.new title = str;

fn.main [] -> void:
    let $s: str = "a";
    let $t: title = $s as title;
    return void;
end fn.main
