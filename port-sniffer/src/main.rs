use std::{env, net::{IpAddr, Ipv4Addr, TcpStream}, str::FromStr, process};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::io::{self, Write};

#[derive(Clone, Debug)] 
enum Flag {
    Help, // requesting help message
    Worker, // requesting a worker to do the job
}

#[derive(Clone, Debug)] 
struct CliArgs {
    flag: Flag,
    thread_count: u16,
    ip_addr: IpAddr,
}

const MAX: u16 = 65535;

//ex
// port-sniffer.exe -h or port-sniffer.exe -help or port-sniffer -j 10 -h 
// if there is a help flag we don't care about the rest and display the help message
// port-sniffer.exe -j 100 192.168.1.1
// the -1 is the thread count
// port-sniffer.exe 192.168.1.1
// accepts a ip v4 or v6 address, must be specified at the end

impl CliArgs {

    fn new(args: &Vec<String>) -> Result<CliArgs, &'static str> {
            match CliArgs::args_check(args) {
                Ok(args) => return CliArgs::args_parse(args),
                Err(e) => return Err(e),
            };
    }

    fn args_check(args: &Vec<String>) -> Result<&Vec<String>, &'static str> {
        if args.len() < 2 {
            return Err("Invalid arguments, too little arguments");
        } else if args.len() > 4 {
            return Err("Invalid arguments, too many arguments");
        }
        
        Ok(args)
    }

    fn args_parse(args: &Vec<String>) -> Result<CliArgs, &'static str> {

        for i in args {
            if i == "-h" || i == "--help" {
                return Ok(CliArgs { flag: Flag::Help, thread_count: 0, ip_addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) });
            }
        }

        if args[1] == "-j" && args.len() == 4 {
            let thread_count: u16 = match args[2].parse::<u16>() {
                Ok(s) => s,
                Err(_) => return Err("failed to parse thread number"),
            };
            let ip_addr = match IpAddr::from_str(&args[3]) {
                Ok(s) => s,
                Err(_) => return Err("not a valid IPADDR; must be IPv4 or IPv6"),
            };
            return Ok(CliArgs {
                flag: Flag::Worker,
                thread_count,
                ip_addr,
            });
        } else if let Ok(ip_addr) = IpAddr::from_str(&args[1]) {
            return Ok(CliArgs {
                flag: Flag::Worker,
                thread_count: 4, // default number of threads is 4
                ip_addr,
            });
        }
        Err("Invalid arguments")
    }

    fn scan(tx: Sender<u16>, start_port: u16, ip_addr: IpAddr, thread_count: u16) {
        let mut port: u16 = start_port + 1;
        loop {
            match TcpStream::connect((ip_addr, port)) {
                Ok(_) => {
                    print!(".");
                    io::stdout().flush().unwrap();
                    tx.send(port).unwrap();
                }
                Err(_) => {
                    //println!("{}: closed port", port);
                }
            }
            if (MAX - port) <= thread_count {
                break;
            }
            port += thread_count;
        }
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    //println!("{:?}", args);
    let program = args[0].clone();
    let cli_args = CliArgs::new(&args).unwrap_or_else(|err| {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(0);
        });
    if let Flag::Help = cli_args.flag {
        println!(
            "Usage: -j to select how many threads you want
        \n\r       -h or -help to show this help message"
        );
        process::exit(0);
    }

    let (tx, rx) = channel();
    let thread_count = cli_args.thread_count;
    let ip_addr = cli_args.ip_addr;
    for i in 0..thread_count {
        let tx = tx.clone();

        thread::spawn(move || {
            CliArgs::scan(tx, i, ip_addr, thread_count);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
// ex to run: cargo run -- -j  1000 192.168.1.1