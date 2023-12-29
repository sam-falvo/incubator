use log::{debug, error};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;

#[derive(Debug)]
pub enum RPCError {
    ConnectError(std::io::Error),
    SessionError(ssh2::Error),
    HandshakeError(ssh2::Error),
    AuthError(ssh2::Error),
}

pub struct RemotePC {
    session: ssh2::Session,
    pub rc: Option<i32>,
    pub result: Option<Arc<String>>,
}

pub enum AuthMethod<'am> {
    UsernamePassword {
        username: &'am str,
        password: &'am str,
    },
}

impl RemotePC {
    pub fn connect(tcp_address: &str, auth_method: AuthMethod) -> Result<RemotePC, RPCError> {
        let stream = TcpStream::connect(tcp_address);
        if let Err(e) = stream {
            error!("TcpStream::connect({:?}) ({:?})", tcp_address, e);
            return Err(RPCError::ConnectError(e));
        }
        let stream = stream.unwrap(); // guaranteed to never fail
        debug!("Connected to {:?}", tcp_address);

        let session = Session::new();
        if let Err(e) = session {
            error!("Session::new() ({:?})", e);
            return Err(RPCError::SessionError(e));
        }
        let mut session = session.unwrap();
        debug!("Session established");

        session.set_tcp_stream(stream);
        let success = session.handshake();
        if let Err(e) = success {
            error!("session.handshake() ({:?})", e);
            return Err(RPCError::HandshakeError(e));
        }
        debug!("Handshook.");

        match auth_method {
            AuthMethod::UsernamePassword { username, password } => {
                debug!("Attempting username/password authentication.");
                let auth = session.userauth_password(username, password);
                if let Err(e) = auth {
                    error!("session.userauth_password({}, \"...\") ({:?})", username, e);
                    return Err(RPCError::AuthError(e));
                }
            }
        }
        debug!("Authenticated.");

        Ok(RemotePC {
            session,
            rc: None,
            result: None,
        })
    }

    pub fn exec(&mut self, command: &str) {
        self.rc = None;
        self.result = None;

        let channel = self.session.channel_session();
        if let Err(e) = channel {
            error!("session.channel_session() ({:?})", e);
            return;
        }
        let mut channel = channel.unwrap();
        debug!("Channel established.");

        let r = channel.exec(command);
        if let Err(e) = r {
            error!("channel.exec(...) ({:?})", e);
            return;
        }

        let mut s = String::new();
        let r = channel.read_to_string(&mut s);
        if let Err(e) = r {
            error!("channel.read_to_string ({:?})", e);
            return;
        }
        debug!("Result: {}", s);
        self.result = Some(Arc::new(s));

        let r = channel.wait_close();
        if let Err(e) = r {
            error!("channel.wait_close() ({:?})", e);
            return;
        }
        debug!("Closed");

        let exit_status = channel.exit_status();
        if let Err(e) = exit_status {
            error!("channel.exit_status() ({:?})", e);
            return;
        }
        let exit_status = exit_status.unwrap();
        debug!("RC={}", exit_status);
        self.rc = Some(exit_status);
    }
}
