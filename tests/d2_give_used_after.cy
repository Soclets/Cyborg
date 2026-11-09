// trace: D2
@config(default);
import std;

struct open_data:
    fd: i32;
end struct open_data

fn send [$f: Given open_data] -> void:
    return void;
end fn send

fn close [$f: Given open_data] -> void:
    return void;
end fn close

fn.main [] -> void:
    let $f: open_data = open_data[fd = 3];
    send(Give $f);
    close(Give $f);
    return void;
end fn.main
