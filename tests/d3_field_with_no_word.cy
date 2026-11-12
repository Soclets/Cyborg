// trace: D3

struct name_tag:
    text: str[len.max: 32 bytes];
end struct name_tag

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "hi";
    let $t: name_tag = name_tag[text = $s];
    return void;
end fn.main
