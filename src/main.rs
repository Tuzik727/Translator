use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream},
    process::exit,
    rc::Rc,
    str::{self, FromStr},
    sync::{Arc, Mutex},
    thread,
    time::Duration, path::Path,
};

use regex::{internal::Program, Regex};
use toml::Table;

struct ProgramData {
    ip: u32,
    port: u16,
    start_ip: u32,
    end_ip: u32,
    start_port: u16,
    end_port: u16,
    log: String,
    list_programu: Arc<Mutex<Vec<(String, SocketAddr)>>>,
}

fn log_zprava(soubor: &String, zprava: String) {
    let path = std::path::Path::new(soubor);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let mut file = File::create(soubor).unwrap();
    write!(file, "{}\n", zprava).unwrap();
}

fn preklad(slovo: &str) -> Option<&str> {
    Some(match slovo {
        "mouse" => "myš",
        "rat" => "krysa",
        "cat" => "kočka",
        "book" => "knížka",
        "knife" => "nůž",
        _ => return None,
    })
}

fn rozdel_prikaz(zprava: &str) -> Option<(String, String)> {
    let pattern = Regex::new(r#"^(TRANSLATEPING|TRANSLATELOCL|TRANSLATESCAN|TRANSLATEPONG|TRANSLATEDSUC|TRANSLATEDERR)"(.*)"$"#).unwrap();

    if let Some(captures) = pattern.captures(zprava) {
        let prikaz_druh = &captures[1];
        let prikaz_payload = &captures[2];

        return Some((prikaz_druh.to_string(), prikaz_payload.to_string()));
    } else {
        return None;
    }
}

fn listen(program_data: Arc<ProgramData>, ip: &str, port: u16) {
    let listener = TcpListener::bind((ip, port)).unwrap();

    log_zprava(&program_data.log,format!("Posloucham na {}, portu {}", ip, port));

    // akceptujeme pripojeni a vytvorime thread
    // filter_map(Result::ok) odstrani chybne pripojeni
    for stream in listener.incoming().filter_map(Result::ok) {
        let program_data_clone = program_data.clone();

        thread::spawn(move || {
            zpracuj_klienta(program_data_clone, stream);
        });
        //self.zpracuj_klienta(stream);
    }
}

fn zpracuj_klienta(program_data: Arc<ProgramData>, mut stream: TcpStream) {
    let mut buffer = [0u8; 256];
    //let mut buffer = String::new();

    while match stream.read(&mut buffer) {
        Ok(size) => {
            let mut zprava = str::from_utf8(&buffer[0..size]).unwrap();

            if zprava != "\r\n" {

                while zprava.chars().last().unwrap().is_whitespace() {
                    zprava = &zprava[0..zprava.len() - 1];
                }

                log_zprava(&program_data.log,
                    format!("{} ({}) -> Server",
                    stream.peer_addr().unwrap(),
                    zprava)
                );

                zpracuj_prikaz(&program_data, &mut stream, zprava);
            }
            true
        }
        Err(chyba) => {
            log_zprava(&program_data.log,
                format!("{} Disconnect", stream.peer_addr().unwrap()));
            false
        }
    } {}
}

fn zpracuj_prikaz(program_data: &Arc<ProgramData>, mut stream: &TcpStream, zprava: &str) {
    if let Some((typ, text)) = rozdel_prikaz(zprava) {
        match typ.as_str() {
            "TRANSLATEPING" => {
                stream
                    .write_all("TRANSLATEPONG\"Butym translator\"".as_bytes())
                    .unwrap();

                log_zprava(&program_data.log, format!("Server ({}) -> {}", "TRANSLATEPONG\"Butym translator\"", stream.peer_addr().unwrap()));
            }
            "TRANSLATELOCL" => {
                if let Some(preklad) = preklad(text.as_str()) {
                    stream
                        .write_all(format!("TRANSLATEDSUC\"{}\"", preklad).as_bytes())
                        .unwrap();
                    
                        log_zprava(&program_data.log, format!("Server ({}) -> {}", format!("TRANSLATEDSUC\"{}\"", preklad), stream.peer_addr().unwrap()));
                } else {
                    stream
                        .write_all("TRANSLATEDERR\"Neznam tohle slovo\"".as_bytes())
                        .unwrap();
                    
                        log_zprava(&program_data.log, format!("Server ({}) -> {}", "TRANSLATEDERR\"Neznam tohle slovo\"", stream.peer_addr().unwrap()));
                }

                skenovani_site(&program_data);
            }
            "TRANSLATESCAN" => {
                skenovani_site(&program_data);

                let list = program_data.list_programu.lock().unwrap();

                for (nazev, adresa) in list.iter() {
                    let connection = TcpStream::connect_timeout(adresa, Duration::from_millis(250)).unwrap();
                    zpracuj_sken(&program_data, &text, &connection, stream);
                }
            }
            "TRANSLATEPONG" => (),
            "TRANSLATEDERR" => (),
            "TRANSLATEDSUC" => (),
            _ => panic!(),
        }
    } else {
        stream
            .write_all("TRANSLATEDERR\"Neznam prikaz\"".as_bytes())
            .unwrap();

            log_zprava(&program_data.log, format!("Server ({}) -> {}", "TRANSLATEDERR\"Neznam prikaz\"", stream.peer_addr().unwrap()));
    }
}

fn zpracuj_sken(program_data: &Arc<ProgramData>, text: &String, mut connection: &TcpStream, mut stream: &TcpStream) {
    connection
        .write_all(format!("TRANSLATELOCL\"{}\"", text).as_bytes())
        .unwrap();

    let mut buffer = [0u8; 256];

    let mut nasli_jsme_preklad = false;

    match connection.read(&mut buffer) {
        Ok(size) => {
            let zprava = str::from_utf8(&buffer[0..size]).unwrap();

            if let Some((typ, response)) = rozdel_prikaz(zprava) {
                match typ.as_str() {
                    "TRANSLATEDSUC" => {
                        log_zprava(&program_data.log,format!("Nasli jsme preklad slova {} u {}", text, connection.peer_addr().unwrap()));

                        stream.write_all(format!("TRANSLATEDSUC\"{}\"", response).as_bytes()).unwrap();
                    
                        nasli_jsme_preklad = true;
                    },
                    _ => ()
                }
            } else {
                stream
                    .write_all("TRANSLATEDERR\"Neznam prikaz\"".as_bytes())
                    .unwrap();
            }

            log_zprava(&program_data.log,format!("Prisla zprava {} od {}",
                zprava,
                stream.peer_addr().unwrap())
            );

        }
        Err(chyba) => {
           log_zprava(&program_data.log,format!("Spojeni ukonceno s {}", stream.peer_addr().unwrap()));
        }
    }

    if !nasli_jsme_preklad {
        stream.write_all(format!("TRANSLATEDERR\"Nenasli jsme zadny preklad slova {}\"", text).as_bytes()).unwrap();
    }
}

fn skenovani_site(program_data: &Arc<ProgramData>) {
    for ip in program_data.start_ip..=program_data.end_ip {
        let program_data_clone = program_data.clone();

        thread::spawn(move || {
            for port in program_data_clone.start_port..=program_data_clone.end_port {
                log_zprava(&program_data_clone.log, format!("{}:{}", Ipv4Addr::from(ip), port));

                match TcpStream::connect_timeout(
                    &SocketAddr::new(IpAddr::V4(ip.into()), port),
                    Duration::from_secs(1),
                ) {
                    Ok(mut stream) => {
                        log_zprava(&program_data_clone.log,format!("Pripojeno k {}:{}", stream.peer_addr().unwrap(), port));

                        zpracuj_peer(&program_data_clone, &mut stream);
                    }
                    Err(_) => (),
                }
            }
        });
    }
}

fn zpracuj_peer(program_data: &Arc<ProgramData>, mut stream: &TcpStream) {
    if stream.peer_addr().unwrap().port() == program_data.port
        && stream.peer_addr().unwrap().ip() == Ipv4Addr::from(program_data.ip)
    {
    } else {
        stream
            .write_all("TRANSLATEPING\"Jsi program?\"".as_bytes())
            .unwrap();

        let mut buffer = [0u8; 256];

        while match stream.read(&mut buffer) {
            Ok(size) => {
                let zprava = str::from_utf8(&buffer[0..size]).unwrap();

                if let Some((typ, text)) = rozdel_prikaz(zprava) {
                    match typ.as_str() {
                        "TRANSLATEPONG" => {
                            let mut list = program_data.list_programu.lock().unwrap();
                            log_zprava(
                                &program_data.log,format!("Pridan program {} s adresou {}",
                                text,
                                stream.peer_addr().unwrap())
                            );
                            list.push((text, stream.peer_addr().unwrap()));
                        }
                        _ => (),
                    }
                } else {
                    stream
                        .write_all("TRANSLATEDERR\"Neznam prikaz\"".as_bytes())
                        .unwrap();
                }

                log_zprava(
                    &program_data.log,format!("Prisla zprava {} od {}",
                    zprava,
                    stream.peer_addr().unwrap())
                );

                true
            }
            Err(chyba) => {
                log_zprava(&program_data.log, format!("Spojeni ukonceno s {}", stream.peer_addr().unwrap()));
                false
            }
        } {}
    }
}

fn main() -> std::process::ExitCode {
    let filename = "conf.toml";

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Config structure error, check your TOML format `{}`", filename);
            exit(1);
        }
    };

    let config = contents.parse::<Table>().expect("Config parse Erorr");

    let ip = config["ip"].as_str().unwrap();
    let port = config["port"].as_integer().unwrap() as u16;

    let program_data = ProgramData {
        ip: Ipv4Addr::from_str(ip).unwrap().into(),
        port: port,
        start_ip: config["start_ip"]
            .as_str()
            .unwrap()
            .parse::<Ipv4Addr>()
            .unwrap()
            .into(),
        end_ip: config["end_ip"]
            .as_str()
            .unwrap()
            .parse::<Ipv4Addr>()
            .unwrap()
            .into(),
        start_port: config["start_port"].as_integer().unwrap() as u16,
        end_port: config["end_port"].as_integer().unwrap() as u16,
        log: config["log"].as_str().unwrap().to_string(),
        list_programu: Arc::new(Vec::new().into()),
    };

    listen(Arc::new(program_data), ip, port);

    return std::process::ExitCode::SUCCESS;
}
