use std::convert::TryInto;

mod clock;


const MAX_SEQ_NUM: u16 = 4095;

#[derive(Debug)]
pub struct Snowflake {
    pub node_id: u16,
    pub seq_num: u16,
    pub ts: u64
}

fn create_id(ts: u64, node: u16, seq: u16) -> u64 {
    let ts_bits = ts << 22;
    let node_bits = (node as u64) << 12;


    ts_bits | node_bits | (seq as u64)
}

pub fn parse_id(id: u64) -> Snowflake {

    let node_id = (id & 0x3FF000) >> 12;
    let seq_num = id & 0xFFF;
    let ts = (id & 0x7FFFFFFFFFC00000) >> 22;

    Snowflake {
        node_id: node_id.try_into().unwrap(),
        seq_num: seq_num.try_into().unwrap(),
        ts
    }
}

impl Snowflake {

    pub fn new(node_id: u16) -> Snowflake {
        assert!(node_id < 1024);
        Snowflake {
            node_id: node_id,
            seq_num: 0,
            ts: 0
        }
    }

    pub fn generate(&mut self) -> u64 {
        let sys_time = clock::get_time();

        if self.ts == sys_time {
            self.seq_num = self.seq_num + 1;
            if self.seq_num > MAX_SEQ_NUM {
                clock::wait();
                self.seq_num = 0;
                self.ts = u64::max(self.ts, clock::get_time());
            } else {
                self.ts = u64::max(self.ts, sys_time);
            }
        } else {
            self.ts = u64::max(self.ts, sys_time);
            self.seq_num = 0;
        }


        create_id(self.ts, self.node_id, self.seq_num)
    }

}

impl Default for Snowflake {
    fn default() -> Self {
        Snowflake {
            node_id: 1,
            seq_num: 0,
            ts: 0
        }
    }
}

impl Iterator for Snowflake {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate())
    }
}



#[cfg(test)]
mod tests {
    use crate::{Snowflake, create_id, parse_id};

    #[test]
    fn round_trip() {
        let mut s = Snowflake::new(1);
        let id = s.generate();
        let result = parse_id(id);
        assert_eq!(s.node_id, result.node_id);
        assert_eq!(s.seq_num, result.seq_num);
        assert_eq!(s.ts, result.ts);
    }

    #[test]
    fn test_create_id() {
        let id = create_id(123456789, 777, 1234);

        assert_eq!(id, 517815307113682);

        let s = parse_id(id);

        assert_eq!(123456789, s.ts);
        assert_eq!(777, s.node_id);
        assert_eq!(1234, s.seq_num);

    }

    #[test]
    fn test_ms_rollover() {
        crate::clock::setup_mock_clock();
        let mut s = Snowflake::new(123);

        let first_id = s.generate();
        let first_id_parsed = parse_id(first_id);

        for seq in 1..4096 {
            let id = s.generate();

            let result = parse_id(id);
            assert_eq!(seq, result.seq_num);
            assert_eq!(first_id_parsed.ts, result.ts);
        }

        let rolled_over_id = s.generate();

        let rolled_parsed = parse_id(rolled_over_id);

        assert_eq!(first_id_parsed.ts + 1, rolled_parsed.ts);
    }


    #[test]
    fn test_many() {
        let mut s = Snowflake::new(5);
        let mut last = s.generate();

        let ids = s.take(1000000);
        for id in ids {
            assert!(id > last, "{} >! {}", id, last);
            last = id;
        }
    }
    
}
