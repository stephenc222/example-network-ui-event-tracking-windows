use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::time::Duration;
use windows::Win32::{
    Foundation::ERROR_INSUFFICIENT_BUFFER,
    Networking::WinSock::AF_INET,
    NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    },
};

use crate::logging::log_message;
use crate::timestamp::{now};

#[derive(Debug, PartialEq, Eq, Hash)]
struct TcpConnection {
    local: (IpAddr, u16),
    remote: (IpAddr, u16),
    pid: u32,
    state: u32,
}

pub fn start_network_monitor() {
    thread::spawn(|| {
        let mut seen: HashSet<TcpConnection> = HashSet::new();

        loop {
            unsafe {
                let mut size = 0u32;
                let result = GetExtendedTcpTable(
                    None,
                    &mut size,
                    false,
                    AF_INET.0.into(),
                    TCP_TABLE_OWNER_PID_ALL,
                    0,
                );

                if result != ERROR_INSUFFICIENT_BUFFER.0 {
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }

                let mut buf = vec![0u8; size as usize];
                let table_ptr = buf.as_mut_ptr() as *mut MIB_TCPTABLE_OWNER_PID;

                let result = GetExtendedTcpTable(
                    Some(table_ptr.cast()),
                    &mut size,
                    false,
                    AF_INET.0.into(),
                    TCP_TABLE_OWNER_PID_ALL,
                    0,
                );

                if result != 0 {
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }

                let table = &*table_ptr;
                let rows = std::slice::from_raw_parts(
                    &table.table[0] as *const MIB_TCPROW_OWNER_PID,
                    table.dwNumEntries as usize,
                );

                let mut current_seen = HashSet::new();

                for row in rows {
                    // Only log if connection is in early or established states
                    match row.dwState {
                        3 | 4 | 5 => {} // SYN_SENT, SYN_RECEIVED, ESTABLISHED
                        _ => continue,
                    }

                    let local = Ipv4Addr::from(u32::from_be(row.dwLocalAddr));
                    let local_port = u16::from_be((row.dwLocalPort >> 8) as u16);
                    let remote = Ipv4Addr::from(u32::from_be(row.dwRemoteAddr));
                    let remote_port = u16::from_be((row.dwRemotePort >> 8) as u16);
                    let pid = row.dwOwningPid;
                    let state = row.dwState;

                    let conn = TcpConnection {
                        local: (IpAddr::V4(local), local_port),
                        remote: (IpAddr::V4(remote), remote_port),
                        pid,
                        state,
                    };

                    if !seen.contains(&conn) {
                        let timestamp = now();
                        let state_name = tcp_state_string(state);

                        log_message(&format!(
                            "[{}] TCP: {}:{} → {}:{}, PID={}, STATE={}",
                            timestamp,
                            conn.local.0,
                            conn.local.1,
                            conn.remote.0,
                            conn.remote.1,
                            conn.pid,
                            state_name
                        ));
                    }

                    current_seen.insert(conn);
                }

                seen = current_seen;
            }

            thread::sleep(Duration::from_millis(20));
        }
    });
}

fn tcp_state_string(state: u32) -> &'static str {
    match state {
        1 => "CLOSED",
        2 => "LISTEN",
        3 => "SYN_SENT",
        4 => "SYN_RECEIVED",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT_1",
        7 => "FIN_WAIT_2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "UNKNOWN",
    }
}
