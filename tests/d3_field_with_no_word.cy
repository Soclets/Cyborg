// trace: D3

struct name_tag:
    text: str;
end struct name_tag

fn.main [none] returns void:
    let $s: str = "hi";
    let $t: name_tag = name_tag[text = $s];
    return void;
end fn.main
