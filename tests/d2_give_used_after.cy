// trace: D2
import std;

struct open_data:
    fd: i32;
end struct open_data

fn send [$f: Given open_data] returns void:
    return void;
end fn send

fn close [$f: Given open_data] returns void:
    return void;
end fn close

fn.main [none] returns void:
    let $f: open_data set_to open_data[fd set_to 3];
    send(Give $f);
    close(Give $f);
    return void;
end fn.main
