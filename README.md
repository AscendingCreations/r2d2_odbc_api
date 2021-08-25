# r2d2-odbc-api
[ODBC](https://github.com/pacman82/odbc-api) adapter for [r2d2](https://github.com/sfackler/r2d2) connection pool

[![https://app.travis-ci.com/github/genusistimelord/r2d2_odbc_api](https://app.travis-ci.com/github/genusistimelord/r2d2_odbc_api.svg?branch=main)](https://app.travis-ci.com/github/genusistimelord/r2d2_odbc_api)
[![https://crates.io/crates/r2d2_odbc_api](https://img.shields.io/badge/crates.io-v0.1.0-blue)](https://crates.io/crates/r2d2_odbc_api)
[![Docs](https://docs.rs/r2d2_odbc_api/badge.svg)](https://docs.rs/r2d2_odbc_api)

Example:

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
    let manager = ODBCConnectionManager::new("DSN=PostgreSQL");
    let pool = r2d2::Pool::new(manager).unwrap();

    let mut children = vec![];
    for i in 0..10i32 {
        let pool = pool.clone();
        children.push(thread::spawn(move || {
            let pool_conn = pool.get().unwrap();
            let conn = pool_conn.raw();

            if let Some(cursor) = conn.execute("SELECT version()", ()).unwrap() {
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
