use clap::Parser;
use env_logger;
use log::{error, info};
use remote_pc::{AuthMethod, RemotePC};

// Clap requires the derive feature.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Commands {
    TestSsh(TestSSHArgs),
    Echo(EchoTest),
}

#[derive(clap::Args, Debug)]
#[command(author, about, long_about = None)]
struct TestSSHArgs {
    #[arg(short, long, help = "Host:Port for remote PC [e.g., example.com:22]")]
    tcp: String,

    #[arg(
        short,
        long,
        default_value = "ls -la",
        help = "Command to invoke on remote PC"
    )]
    command: String,

    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    password: String,
}

#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = None)]
struct EchoTest {}

fn main() {
    env_logger::init();

    let args = Commands::parse();
    match args {
        Commands::TestSsh(args) => {
            let auth_method = AuthMethod::UsernamePassword {
                username: &args.username,
                password: &args.password,
            };
            let rpc = RemotePC::connect(&args.tcp, auth_method);
            if let Err(e) = rpc {
                error!("RemotePC::connect() ({:?})", e);
                return;
            }
            let mut rpc = rpc.unwrap();

            rpc.exec(&args.command);
            match rpc.rc {
                None => info!("Command execution failed somehow; set RUST_LOG to debug"),
                Some(rc) => {
                    info!("Command returned {}", rc);

                    match rpc.result {
                        None => info!("No output generated"),
                        Some(results) => info!("Command results: {}", *results),
                    }
                }
            }
        }

        Commands::Echo(_) => {
            info!("Hello world; Clap crate is working OK");
            return;
        }
    }
}

mod remote_pc;
