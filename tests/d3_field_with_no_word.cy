// trace: D3

struct name_tag:
    text: str;
end struct name_tag

fn.main [none] returns void:
    let $s: str set_to "hi";
    let $t: name_tag set_to name_tag[text set_to $s];
    return void;
end fn.main
