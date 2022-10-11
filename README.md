<h1 align="center">
    r2d2-odbc-api
</h1>
<div align="center">
    <a href="https://github.com/pacman82/odbc-api">[ODBC]</a> adapter for <a href="https://github.com/sfackler/r2d2">[r2d2]</a> connection pool with custom Pooling.
</div>
<br />
<div align="center">
    <a href="https://crates.io/crates/r2d2_odbc_api"><img src="https://img.shields.io/crates/v/r2d2_odbc_api?style=plastic" alt="crates.io"></a>
    <a href="https://docs.rs/r2d2_odbc_api"><img src="https://docs.rs/r2d2_odbc_api/badge.svg" alt="docs.rs"></a>
    <img src="https://img.shields.io/badge/min%20rust-1.60-green.svg" alt="Minimum Rust Version">
</div>

## License

This project is licensed under either [Apache License, Version 2.0](LICENSE-APACHE), [zlib License](LICENSE-ZLIB), or [MIT License](LICENSE-MIT), at your option.


## Example

```rust

extern crate anyhow;
extern crate odbc_api;
extern crate r2d2;
extern crate r2d2_odbc_api;

use anyhow::Error;
use odbc_api::*;
use r2d2_odbc_api::ODBCConnectionManager;
use std::str;
use std::thread;

fn main() -> Result<(), Error> {
    let max_pool_size = 10;
    let manager = ODBCConnectionManager::new("DSN=PostgreSQL", max_pool_size);
    let pool = r2d2::Pool::builder()
        .max_size(max_pool_size)
        .build(manager)
        .unwrap();

    let mut children = vec![];
    for i in 0..10i32 {
        let pool = pool.clone();
        children.push(thread::spawn(move || {
            let pool_conn = pool.get().unwrap();

            if let Some(cursor) = pool_conn.execute("SELECT version()", ()).unwrap() {
                let mut buffers =
                    buffers::TextRowSet::for_cursor(5000, &cursor, Some(4096)).unwrap();
                let mut row_set_cursor = cursor.bind_buffer(&mut buffers).unwrap();

                while let Some(batch) = row_set_cursor.fetch().unwrap() {
                    if let Some(val) = batch.at(0, 0) {
                        println!("THREAD {} {}", i, str::from_utf8(val).unwrap());
                    }
                }
            };
        }));
    }

    for child in children {
        let _ = child.join();
    }

    Ok(())
}

```