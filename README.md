# snowflake-rs

This is a Rust implementation of Twitter's Snowflake ID's, which generate unique 64-bit ID's in a distributed manner.
The ID's generated are roughly time sortable and made up of a 41 bit timestamp, 10 bit node ID, and a 12 bit sequence number.

## Usage
    let mut s = Snowflake::new(5);
    let mut last = s.generate();
