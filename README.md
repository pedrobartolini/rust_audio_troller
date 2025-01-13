## Usage
change these consts in src/bin/play_server.rs

pub const PUBLIC_IP: &str = "127.0.0.1";

pub const PORT: u16 = 6969;


### Build with
```
cargo install --path .
```

### Creating service in windows
```
sc.exe create [service_name] binPath= "..../play_client.exe" start= auto
```


### Send audio using
```
play_server -file [path:string] -clients [play_server clients:number]
```








