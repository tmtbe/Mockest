//! Sonyflake 雪花算法
//! 结构
//! |--闲置位--|--时间戳（ms）--|--标志--|--序号--|
//! |-- 1位 --|---- 41位 -----|--10位--|--12位--|
//!

use std::net::UdpSocket;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::SystemTime;

pub struct SonyFlakeEntity {
    time_stamp: AtomicU64,
    counter: AtomicU32,
    node_id: u16,
}

impl SonyFlakeEntity {
    /// 根据ip地址设置节点id，如果获取失败则设置为1
    pub fn new_default() -> Self {
        let mut nodeid = 1;
        if let Some(s) = get_ip() {
            let ips: Vec<_> = s.split('.').collect();
            let a = ips[2].parse::<u8>().unwrap();
            let b = ips[3].parse::<u8>().unwrap();
            nodeid = u16::from_be_bytes([a, b]);
        }
        SonyFlakeEntity {
            time_stamp: AtomicU64::new(0),
            counter: AtomicU32::new(0),
            node_id: nodeid,
        }
    }

    pub fn get_id(&self, system_time: SystemTime) -> i64 {
        let (ts, sid) = self.get_ts_sid(system_time);
        return Self::generate_id(ts, sid, self.node_id);
    }

    #[allow(unused_assignments)]
    fn get_ts_sid(&self, system_time: SystemTime) -> (u64, u32) {
        let mut timestamp = 0;
        let mut sequence: u32 = 0;
        let mut system_time_u64: u128 = 0;
        match system_time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => system_time_u64 = n.as_millis(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
        let ntime = system_time_u64 as u64;
        let otime = self.time_stamp.load(Ordering::Acquire);
        if ntime <= otime {
            sequence = self.counter.fetch_add(1, Ordering::Relaxed);
            timestamp = otime; //self.time_stamp.load(Ordering::Release);
        } else {
            self.counter.store(1, Ordering::Relaxed);
            timestamp = ntime;
            self.time_stamp.store(ntime, Ordering::Release);
        }
        return (timestamp, sequence);
    }
    fn generate_id(ts: u64, sid: u32, nid: u16) -> i64 {
        let mut res: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let ts = ts << 22;
        let vts = Vec::from(ts.to_be_bytes());
        for i in 0..6 {
            res[i] = res[i] | vts[i];
        }
        res[0] = res[0] & 0x7F;
        // let nid = nid << 4;
        let mut vnid = Vec::from(nid.to_be_bytes());
        vnid[0] = vnid[0] & 0x03;
        res[5] = res[5] | vnid[0];
        res[6] = res[6] | vnid[1];
        let mut vsid = Vec::from(sid.to_be_bytes());
        vsid[2] = vsid[2] & 0x0F;
        res[6] = res[6] | vsid[2];
        res[7] = res[7] | vsid[3];
        i64::from_be_bytes([
            res[0], res[1], res[2], res[3], res[4], res[5], res[6], res[7],
        ])
    }
}

pub fn get_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => return Some(addr.ip().to_string()),
        Err(_) => return None,
    };
}
